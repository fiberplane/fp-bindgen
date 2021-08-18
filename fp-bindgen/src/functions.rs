use crate::{prelude::Primitive, types::Type};
use quote::ToTokens;
use std::{collections::BTreeSet, str::FromStr};
use syn::{FnArg, ForeignItemFn, ReturnType};

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
    pub args: Vec<FunctionArg>,
    pub return_type: Option<Type>,
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
                    ty: resolve_type(arg.ty.as_ref(), serializable_types)
                        .expect("Function argument type was not among the serializable types"),
                },
            })
            .collect();
        let return_type = match &item.sig.output {
            ReturnType::Default => None,
            ReturnType::Type(_, return_type) => Some(
                resolve_type(return_type.as_ref(), deserializable_types)
                    .expect("Function return type was not among the deserializable types"),
            ),
        };
        let is_async = item.sig.asyncness.is_some();

        Self {
            name,
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
    pub ty: Type,
}

/// Resolves a type based on its type path and a set of user-defined types to match against.
fn resolve_type(ty: &syn::Type, types: &BTreeSet<Type>) -> Option<Type> {
    match ty {
        syn::Type::Path(path) if path.qself.is_none() => {
            let path = path.path.to_token_stream().to_string();
            match Primitive::from_str(&path) {
                Ok(primitive) => Some(Type::Primitive(primitive)),
                Err(_) => types.iter().find(|ty| ty.name() == path).cloned(),
            }
        }
        _ => panic!(
            "Only value types are supported. Incompatible type: {:?}",
            ty
        ),
    }
}
