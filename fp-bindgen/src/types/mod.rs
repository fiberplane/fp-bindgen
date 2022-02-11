use crate::{primitives::Primitive, serializable::Serializable};
use fp_bindgen_support::common::errors::FPGuestError;
use quote::{quote, ToTokens};
use std::{collections::BTreeMap, hash::Hash};
use syn::Item;

mod cargo_dependency;
mod custom_type;
mod enums;
mod structs;
mod type_ident;

pub use cargo_dependency::CargoDependency;
pub use custom_type::CustomType;
pub use enums::{Enum, EnumOptions, Variant, VariantAttrs};
pub use structs::{Field, FieldAttrs, Struct, StructOptions};
pub use type_ident::TypeIdent;

pub type TypeMap = BTreeMap<TypeIdent, Type>;

pub fn create_default_type_map() -> TypeMap {
    TypeMap::from([
        //Always add Result and GuestError types to the generated types
        //since they are relied on for ferrying errors out of wasm land
        (FPGuestError::ident(), FPGuestError::ty()),
        //The actual T and E of Result doesn't matter here
        (Result::<i32, i32>::ident(), Result::<i32, i32>::ty())
    ])
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    Alias(String, TypeIdent),
    Container(String, TypeIdent),
    Custom(CustomType),
    Enum(Enum),
    List(String, TypeIdent),
    Map(String, TypeIdent, TypeIdent),
    Primitive(Primitive),
    String,
    Struct(Struct),
    Tuple(Vec<TypeIdent>),
    Unit,
}

impl Type {
    pub fn from_item(item_str: &str) -> Self {
        let item = syn::parse_str::<Item>(item_str).unwrap();
        match item {
            Item::Enum(item) => Type::Enum(enums::parse_enum_item(item)),
            Item::Struct(item) => Type::Struct(structs::parse_struct_item(item)),
            item => panic!(
                "Only struct and enum types can be constructed from an item. Found: {:?}",
                item
            ),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Alias(name, _) => name.clone(),
            Self::Container(name, ident) => format!("{}<{}>", name, ident),
            Self::Custom(custom) => custom.ident.to_string(),
            Self::Enum(Enum { ident, .. }) => ident.to_string(),
            Self::List(name, ident) => format!("{}<{}>", name, ident),
            Self::Map(name, key, value) => format!("{}<{}, {}>", name, key, value),
            Self::Primitive(primitive) => primitive.name(),
            Self::String => "String".to_owned(),
            Self::Struct(Struct { ident, .. }) => ident.to_string(),
            Self::Tuple(items) => format!(
                "({})",
                items
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Unit => "()".to_owned(),
        }
    }
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        (match self {
            Type::Alias(name, _) | Type::Custom(CustomType { rs_ty: name, .. }) => {
                let ty = syn::parse_str::<syn::Type>(name).unwrap();
                quote! { #ty }
            }
            Type::Container(name, ident) | Type::List(name, ident) => {
                let name = syn::parse_str::<syn::Type>(name).unwrap();
                quote! { #name<#ident> }
            }
            Type::Struct(Struct { ident, .. }) | Type::Enum(Enum { ident, .. }) => {
                quote! { #ident }
            }
            Type::Map(name, k, v) => {
                let name = syn::parse_str::<syn::Type>(name).unwrap();
                quote! { #name<#k, #v> }
            }
            Type::Primitive(primitive) => quote! { #primitive },
            Type::String => quote! { String },
            Type::Tuple(items) => quote! { (#(#items),*) },
            Type::Unit => quote! { () },
        })
        .to_tokens(tokens)
    }
}
