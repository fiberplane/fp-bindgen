use crate::utils::normalize_return_type;
use crate::{docs::get_doc_lines, types::TypeIdent};
use quote::ToTokens;
use std::{collections::BTreeSet, convert::TryFrom};
use syn::{FnArg, ForeignItemFn};

/// Maps from function name to the stringified function declaration.
#[derive(Debug, Default)]
pub struct FunctionList(BTreeSet<Function>);

impl FunctionList {
    pub fn add_function(&mut self, function_decl: &str) {
        self.0.insert(Function::new(function_decl));
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
    pub fn new(decl: &str) -> Self {
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
        let return_type = normalize_return_type(&item.sig.output).map(|return_type| {
            TypeIdent::try_from(return_type)
                .unwrap_or_else(|_| panic!("Invalid return type for function {}", name))
        });
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

#[derive(Debug, Eq, PartialEq)]
pub struct FunctionArg {
    pub name: String,
    pub ty: TypeIdent,
}
