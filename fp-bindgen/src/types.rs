use crate::primitives::Primitive;
use std::{collections::BTreeSet, str::FromStr};
use syn::{GenericArgument, Item, ItemEnum, ItemStruct, PathArguments};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Enum(String, Vec<Variant>),
    List(String, Box<Type>),
    Map(String, Box<Type>, Box<Type>),
    Option(Box<Type>),
    Primitive(Primitive),
    String,
    Struct(String, Vec<Field>),
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
            Self::Enum(name, _) => name.clone(),
            Self::List(name, ty) => format!("{}<{}>", name, ty.name()),
            Self::Map(name, key, value) => format!("{}<{}, {}>", name, key.name(), value.name()),
            Self::Option(ty) => format!("Option<{}>", ty.name()),
            Self::Primitive(primitive) => primitive.name(),
            Self::String => "String".to_owned(),
            Self::Struct(name, _) => name.clone(),
        }
    }

    /// Whether the type is transparent. Transparent types don't need their own type definition,
    /// because they can be represented at the language level or by the standard library, but they
    /// do have dependencies (generic arguments).
    pub fn is_transparent(&self) -> bool {
        match self {
            Self::List(_, _) | Self::Map(_, _, _) | Self::Option(_) => true,
            Self::Enum(_, _) | Self::Primitive(_) | Self::String | Self::Struct(_, _) => false,
        }
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

fn parse_enum_item(item: ItemEnum, dependencies: &BTreeSet<Type>) -> Type {
    let name = item.ident.to_string();
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
            let fields = variant
                .fields
                .iter()
                .map(|field| {
                    let name = field
                        .ident
                        .as_ref()
                        .expect("Enum variant fields must be named")
                        .to_string();
                    let ty = resolve_type(&field.ty, dependencies).unwrap_or_else(|| {
                        panic!("Unresolvable variant field type: {:?}", field.ty)
                    });
                    Field { name, ty }
                })
                .collect();
            let ty = Type::Struct(name.clone(), fields);
            Variant { name, ty }
        })
        .collect();
    Type::Enum(name, variants)
}

fn parse_struct_item(item: ItemStruct, dependencies: &BTreeSet<Type>) -> Type {
    let name = item.ident.to_string();
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
    Type::Struct(name, fields)
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
                                GenericArgument::Type(ty) => resolve_type(ty, types),
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
                        Type::Enum(name, _) => name == &path_without_args,
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
                        Type::Struct(name, _) => name == &path_without_args,
                    })
                    .cloned(),
            }
        }
        _ => panic!(
            "Only value types are supported. Incompatible type: {:?}",
            ty
        ),
    }
}
