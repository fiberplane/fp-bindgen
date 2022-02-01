use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::{convert::TryFrom, fmt::Display};
use syn::{PathArguments, TypePath};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TypeIdent {
    pub name: String,
    pub generic_args: Vec<TypeIdent>,
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

impl From<String> for TypeIdent {
    fn from(name: String) -> Self {
        Self {
            name,
            generic_args: Vec::new(),
        }
    }
}

impl ToTokens for TypeIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = syn::parse_str::<syn::Type>(&self.name).unwrap();
        if self.generic_args.is_empty() {
            quote! { #name }
        } else {
            let args = self.generic_args;
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
                    .unwrap_or_else(Vec::new),
            }),
            ty => Err(format!("Unsupported type: {:?}", ty)),
        }
    }
}
