use crate::primitives::Primitive;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::{
    convert::{Infallible, TryFrom},
    fmt::Display,
    str::FromStr,
};
use syn::{PathArguments, TypePath};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TypeIdent {
    pub name: String,
    pub generic_args: Vec<TypeIdent>,
}

impl TypeIdent {
    pub fn is_primitive(&self) -> bool {
        Primitive::from_str(&self.name).is_ok()
    }
}

impl Display for TypeIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.generic_args.is_empty() {
            f.write_str(&self.name)
        } else {
            f.write_fmt(format_args!(
                "{}<{}>",
                self.name,
                self.generic_args
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        }
    }
}

impl From<&str> for TypeIdent {
    fn from(name: &str) -> Self {
        Self::from(name.to_owned())
    }
}

impl FromStr for TypeIdent {
    type Err = Infallible;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(start_index) = string.find('<') {
            let end_index = string.rfind('>').unwrap_or(string.len());
            Ok(Self {
                name: string[0..start_index]
                    .trim_end_matches(|c: char| c.is_whitespace() || c == ':')
                    .to_owned(),
                generic_args: string[start_index + 1..end_index]
                    .split(',')
                    .into_iter()
                    .map(|arg| Self::from_str(arg.trim()))
                    .collect::<Result<Vec<Self>, Self::Err>>()?,
            })
        } else {
            Ok(Self::from(string))
        }
    }
}

impl From<String> for TypeIdent {
    fn from(name: String) -> Self {
        Self {
            name,
            generic_args: Vec::new(),
        }
    }
}

impl Ord for TypeIdent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // We only compare the name so that any type is only included once in
        // a map, regardless of how many concrete instances are used with
        // different generic arguments.
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for TypeIdent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // We only compare the name so that any type is only included once in
        // a map, regardless of how many concrete instances are used with
        // different generic arguments.
        self.name.partial_cmp(&other.name)
    }
}

impl ToTokens for TypeIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = syn::parse_str::<syn::Type>(&self.name).unwrap();
        if self.generic_args.is_empty() {
            quote! { #name }
        } else {
            let args = &self.generic_args;
            quote! { #name<#(#args),*> }
        }
        .to_tokens(tokens)
    }
}

impl TryFrom<&syn::Type> for TypeIdent {
    type Error = String;

    fn try_from(ty: &syn::Type) -> Result<Self, Self::Error> {
        match ty {
            syn::Type::Path(TypePath { path, qself }) if qself.is_none() => Ok(Self {
                name: path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::"),
                generic_args: path
                    .segments
                    .last()
                    .and_then(|segment| match &segment.arguments {
                        PathArguments::AngleBracketed(args) => Some(
                            args.args
                                .iter()
                                .flat_map(|arg| match arg {
                                    syn::GenericArgument::Type(ty) => TypeIdent::try_from(ty),
                                    arg => Err(format!("Unsupported generic argument: {:?}", arg)),
                                })
                                .collect(),
                        ),
                        _ => None,
                    })
                    .unwrap_or_default(),
            }),
            ty => Err(format!("Unsupported type: {:?}", ty)),
        }
    }
}
