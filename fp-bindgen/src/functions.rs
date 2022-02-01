use crate::{
    docs::get_doc_lines,
    types::{Type, TypeIdent},
};
use quote::{format_ident, quote, ToTokens};
use std::{collections::BTreeSet, convert::TryFrom};
use syn::{token::Async, FnArg, ForeignItemFn, ReturnType};

/// Maps from function name to the stringified function declaration.
#[derive(Debug, Default)]
pub struct FunctionList(BTreeSet<Function>);

impl FunctionList {
    pub fn add_function(
        &mut self,
        function_decl: &str,
        serializable_types: &BTreeSet<Type>,
        deserializable_types: &BTreeSet<Type>,
    ) {
        self.0.insert(Function::new(
            function_decl,
            serializable_types,
            deserializable_types,
        ));
    }

    pub fn iter(&self) -> std::collections::btree_set::Iter<'_, Function> {
        self.0.iter()
    }

    pub fn new() -> Self {
        Self(BTreeSet::new())
    }
}

impl IntoIterator for FunctionList {
    type Item = Function;
    type IntoIter = std::collections::btree_set::IntoIter<Function>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a FunctionList {
    type Item = &'a Function;
    type IntoIter = std::collections::btree_set::Iter<'a, Function>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Function {
    pub name: String,
    pub doc_lines: Vec<String>,
    pub args: Vec<FunctionArg>,
    pub return_type: Option<TypeIdent>,
    pub is_async: bool,
}

impl Function {
    pub fn new(
        decl: &str,
        serializable_types: &BTreeSet<Type>,
        deserializable_types: &BTreeSet<Type>,
    ) -> Self {
        let item =
            syn::parse_str::<ForeignItemFn>(decl).expect("Cannot parse function declaration");

        let name = item.sig.ident.to_string();
        let doc_lines = get_doc_lines(&item.attrs);
        let args = item
            .sig
            .inputs
            .iter()
            .map(|input| match input {
                FnArg::Receiver(_) => panic!(
                    "Methods are not supported. Found `self` in function declaration: {:?}",
                    item
                ),
                FnArg::Typed(arg) => FunctionArg {
                    name: arg.pat.to_token_stream().to_string(),
                    ty: TypeIdent::try_from(arg.ty.as_ref())
                        .unwrap_or_else(|_| panic!("Invalid argument type for function {}", name)),
                },
            })
            .collect();
        let return_type = match &item.sig.output {
            ReturnType::Default => None,
            ReturnType::Type(_, return_type) => Some(
                TypeIdent::try_from(return_type.as_ref())
                    .unwrap_or_else(|_| panic!("Invalid return type for function {}", name)),
            ),
        };
        let is_async = item.sig.asyncness.is_some();

        Self {
            name,
            doc_lines,
            args,
            return_type,
            is_async,
        }
    }
}

impl Ord for Function {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Function {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl ToTokens for Function {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            name,
            args,
            doc_lines,
            is_async,
            return_type,
        } = self;

        let name = format_ident!("{}", name);

        let asyncness = is_async.then(|| Async {
            ..Default::default()
        });

        (quote! {
            #(#[doc = #doc_lines])*
            #asyncness fn #name(#(#args),*) -> #return_type
        })
        .to_tokens(tokens);
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct FunctionArg {
    pub name: String,
    pub ty: TypeIdent,
}

impl ToTokens for FunctionArg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { name, ty } = self;
        let name = format_ident!("{}", name);

        (quote! { #name: #ty }).to_tokens(tokens)
    }
}

#[cfg(test)]
mod test {
    use super::{Function, FunctionArg};
    use crate::types::TypeIdent;
    use quote::ToTokens;

    #[test]
    fn test_function_arg_to_tokens() {
        let arg = FunctionArg {
            name: "foobar".into(),
            ty: TypeIdent::from("i64".to_owned()),
        };

        let stringified = arg.into_token_stream().to_string();

        pretty_assertions::assert_eq!(&stringified, "foobar : i64");
    }

    #[test]
    fn test_function_to_tokens() {
        let func = Function {
            name: "foobar".into(),
            is_async: false,
            doc_lines: vec![],
            return_type: Some(TypeIdent::from("String".to_owned())),
            args: vec![FunctionArg {
                name: "a1".into(),
                ty: TypeIdent::from("u64".to_owned()),
            }],
        };

        let string = func.into_token_stream().to_string();

        pretty_assertions::assert_eq!(&string, "fn foobar (a1 : u64) -> String");
    }
}
