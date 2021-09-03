use crate::primitives::Primitive;
use quote::ToTokens;
use std::{collections::BTreeSet, str::FromStr};
use syn::{
    ext::IdentExt, parenthesized, parse::Parse, parse::ParseStream, Attribute, Error, GenericParam,
    Ident, Item, ItemEnum, ItemStruct, LitStr, PathArguments, Result, Token,
};

/// A generic argument has a name (T, E, ...) and an optional type, which is only known in contexts
/// when we are dealing with concrete instances of the generic type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericArgument {
    pub name: String,
    pub ty: Option<Type>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Enum(String, Vec<GenericArgument>, Vec<Variant>, EnumOptions),
    GenericArgument(Box<GenericArgument>),
    List(String, Box<Type>),
    Map(String, Box<Type>, Box<Type>),
    Option(Box<Type>),
    Primitive(Primitive),
    String,
    Struct(String, Vec<GenericArgument>, Vec<Field>),
    Tuple(Vec<Type>),
    Unit,
}

impl Type {
    pub fn from_item(item_str: &str, dependencies: &BTreeSet<Type>) -> Self {
        let item = syn::parse_str::<Item>(item_str).unwrap();
        match item {
            Item::Enum(item) => parse_enum_item(item, dependencies),
            Item::Struct(item) => parse_struct_item(item, dependencies),
            item => panic!(
                "Only struct and enum types can be constructed from an item. Found: {:?}",
                item
            ),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Enum(name, generic_args, _, _) => format_name_with_generics(name, generic_args),
            Self::GenericArgument(arg) => arg.name.clone(),
            Self::List(name, ty) => format!("{}<{}>", name, ty.name()),
            Self::Map(name, key, value) => format!("{}<{}, {}>", name, key.name(), value.name()),
            Self::Option(ty) => format!("Option<{}>", ty.name()),
            Self::Primitive(primitive) => primitive.name(),
            Self::String => "String".to_owned(),
            Self::Struct(name, generic_args, _) => format_name_with_generics(name, generic_args),
            Self::Tuple(items) => format!(
                "({})",
                items
                    .iter()
                    .map(|item| item.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Unit => "()".to_owned(),
        }
    }

    pub fn named_generic(name: &str) -> Self {
        Self::GenericArgument(Box::new(GenericArgument {
            name: name.to_owned(),
            ty: None,
        }))
    }
}

impl Ord for Type {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name().cmp(&other.name())
    }
}

impl PartialOrd for Type {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name().partial_cmp(&other.name())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnumOptions {
    pub content_prop_name: Option<String>,
    pub tag_prop_name: Option<String>,
}

impl EnumOptions {
    pub fn to_serde_attrs(&self) -> Vec<String> {
        let mut serde_attrs = vec!["rename_all = \"snake_case\"".to_owned()];
        if let Some(prop_name) = &self.tag_prop_name {
            serde_attrs.push(format!("tag = {}", prop_name));

            if let Some(prop_name) = &self.content_prop_name {
                serde_attrs.push(format!("content = {}", prop_name))
            }
        }
        serde_attrs
    }
}

impl Parse for EnumOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);

        let mut result = Self::default();
        loop {
            let key: Ident = content.call(IdentExt::parse_any)?;
            match &*key.to_string() {
                "content" => {
                    content.parse::<Token![=]>()?;
                    result.content_prop_name =
                        Some(content.parse::<LitStr>()?.to_token_stream().to_string());
                }
                "tag" => {
                    content.parse::<Token![=]>()?;
                    result.tag_prop_name =
                        Some(content.parse::<LitStr>()?.to_token_stream().to_string());
                }
                other => {
                    return Err(Error::new(
                        content.span(),
                        format!("Unexpected attribute: {}", other),
                    ))
                }
            }

            if content.is_empty() {
                break;
            }

            content.parse::<Token![,]>()?;
        }

        Ok(result)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Variant {
    pub name: String,
    pub ty: Type,
}

pub fn format_name_with_generics(name: &str, generic_args: &[GenericArgument]) -> String {
    if generic_args.is_empty() {
        name.to_owned()
    } else {
        format!(
            "{}<{}>",
            name,
            generic_args
                .iter()
                .map(|arg| arg.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn parse_enum_item(item: ItemEnum, dependencies: &BTreeSet<Type>) -> Type {
    let name = item.ident.to_string();
    let generic_args = item
        .generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(ty) => Some(GenericArgument {
                name: ty.ident.to_string(),
                ty: None,
            }),
            _ => None,
        })
        .collect();
    let variants = item
        .variants
        .iter()
        .map(|variant| {
            if variant.discriminant.is_some() {
                panic!(
                    "Discriminants in enum variants are not supported. Found: {:?}",
                    item
                );
            }

            let name = variant.ident.to_string();
            let ty = if variant.fields.is_empty() {
                Type::Unit
            } else if variant.fields.iter().any(|field| field.ident.is_some()) {
                let fields = variant
                    .fields
                    .iter()
                    .map(|field| {
                        let name = field
                            .ident
                            .as_ref()
                            .expect("Expected all enum variant fields to be named")
                            .to_string();
                        let ty = resolve_type(&field.ty, dependencies).unwrap_or_else(|| {
                            panic!("Unresolvable variant field type: {:?}", field.ty)
                        });
                        Field { name, ty }
                    })
                    .collect();
                Type::Struct(name.clone(), vec![], fields)
            } else {
                let item_types = variant
                    .fields
                    .iter()
                    .map(|field| {
                        resolve_type(&field.ty, dependencies).unwrap_or_else(|| {
                            panic!("Unresolvable variant item type: {:?}", field.ty)
                        })
                    })
                    .collect();
                Type::Tuple(item_types)
            };

            Variant { name, ty }
        })
        .collect();
    let opts = parse_enum_options(&item.attrs);
    Type::Enum(name, generic_args, variants, opts)
}

fn parse_enum_options(attrs: &[Attribute]) -> EnumOptions {
    attrs
        .iter()
        .find(|attr| attr.path.is_ident("fp"))
        .map(|attr| {
            syn::parse2::<EnumOptions>(attr.tokens.clone()).expect("Could not parse attributes")
        })
        .unwrap_or_else(EnumOptions::default)
}

fn parse_struct_item(item: ItemStruct, dependencies: &BTreeSet<Type>) -> Type {
    let name = item.ident.to_string();
    let generic_args = item
        .generics
        .params
        .into_iter()
        .filter_map(|param| match param {
            GenericParam::Type(ty) => Some(GenericArgument {
                name: ty.ident.to_string(),
                ty: None,
            }),
            _ => None,
        })
        .collect();
    let fields = item
        .fields
        .iter()
        .map(|field| {
            let name = field
                .ident
                .as_ref()
                .expect("Struct fields must be named")
                .to_string();
            let ty = resolve_type(&field.ty, dependencies)
                .unwrap_or_else(|| panic!("Unresolvable field type: {:?}", field.ty));
            Field { name, ty }
        })
        .collect();
    Type::Struct(name, generic_args, fields)
}

/// Resolves a type based on its type path and a set of user-defined types to match against.
pub fn resolve_type(ty: &syn::Type, types: &BTreeSet<Type>) -> Option<Type> {
    match ty {
        syn::Type::Path(path) if path.qself.is_none() => {
            let path_without_args = path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            let type_args: Vec<Type> = path
                .path
                .segments
                .last()
                .and_then(|segment| match &segment.arguments {
                    PathArguments::AngleBracketed(args) => Some(
                        args.args
                            .iter()
                            .flat_map(|arg| match arg {
                                syn::GenericArgument::Type(ty) => resolve_type(ty, types),
                                _ => None,
                            })
                            .collect(),
                    ),
                    _ => None,
                })
                .unwrap_or_else(Vec::new);
            match Primitive::from_str(&path_without_args) {
                Ok(primitive) => Some(Type::Primitive(primitive)),
                Err(_) => types
                    .iter()
                    .find(|ty| match ty {
                        Type::Enum(name, generic_args, _, _) => {
                            name == &path_without_args
                                && generic_args
                                    .iter()
                                    .filter_map(|arg| arg.ty.clone())
                                    .collect::<Vec<_>>()
                                    == type_args
                        }
                        Type::GenericArgument(arg) => {
                            arg.name == path_without_args && type_args.is_empty()
                        }
                        Type::List(name, ty) => {
                            name == &path_without_args
                                && type_args
                                    .first()
                                    .map(|arg| arg == ty.as_ref())
                                    .unwrap_or(false)
                        }
                        Type::Map(name, key, value) => {
                            name == &path_without_args
                                && type_args
                                    .first()
                                    .map(|arg| arg == key.as_ref())
                                    .unwrap_or(false)
                                && type_args
                                    .get(1)
                                    .map(|arg| arg == value.as_ref())
                                    .unwrap_or(false)
                        }
                        Type::Option(ty) => {
                            path_without_args == "Option"
                                && type_args
                                    .first()
                                    .map(|arg| arg == ty.as_ref())
                                    .unwrap_or(false)
                        }
                        Type::Primitive(primitive) => primitive.name() == path_without_args,
                        Type::String => path_without_args == "String",
                        Type::Struct(name, generic_args, _) => {
                            name == &path_without_args
                                && generic_args
                                    .iter()
                                    .filter_map(|arg| arg.ty.clone())
                                    .collect::<Vec<_>>()
                                    == type_args
                        }
                        Type::Tuple(_) => false,
                        Type::Unit => false,
                    })
                    .cloned(),
            }
        }
        syn::Type::Tuple(tuple) => {
            let item_types = tuple
                .elems
                .iter()
                .map(|ty| {
                    resolve_type(ty, types)
                        .unwrap_or_else(|| panic!("Unresolvable type in tuple: {:?}", ty))
                })
                .collect::<Vec<_>>();
            if item_types.is_empty() {
                types.iter().find(|&ty| ty == &Type::Unit).cloned()
            } else {
                types
                    .iter()
                    .find(|ty| matches!(ty, Type::Tuple(items) if items == &item_types))
                    .cloned()
            }
        }
        _ => panic!(
            "Only value types are supported. Incompatible type: {:?}",
            ty
        ),
    }
}
