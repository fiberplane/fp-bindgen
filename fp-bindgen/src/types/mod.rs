use crate::primitives::Primitive;
use std::{collections::BTreeSet, str::FromStr};
use syn::{Item, PathArguments};

mod enums;
mod structs;

pub use enums::{EnumOptions, Variant};
pub use structs::{Field, StructOptions};

/// A generic argument has a name (T, E, ...) and an optional type, which is only known in contexts
/// when we are dealing with concrete instances of the generic type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericArgument {
    pub name: String,
    pub ty: Option<Type>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Alias(String, Box<Type>),
    Container(String, Box<Type>),
    Custom(CustomType),
    Enum(String, Vec<GenericArgument>, Vec<Variant>, EnumOptions),
    GenericArgument(Box<GenericArgument>),
    List(String, Box<Type>),
    Map(String, Box<Type>, Box<Type>),
    Primitive(Primitive),
    String,
    Struct(String, Vec<GenericArgument>, Vec<Field>, StructOptions),
    Tuple(Vec<Type>),
    Unit,
}

impl Type {
    pub fn from_item(item_str: &str, dependencies: &BTreeSet<Type>) -> Self {
        let item = syn::parse_str::<Item>(item_str).unwrap();
        match item {
            Item::Enum(item) => enums::parse_enum_item(item, dependencies),
            Item::Struct(item) => structs::parse_struct_item(item, dependencies),
            item => panic!(
                "Only struct and enum types can be constructed from an item. Found: {:?}",
                item
            ),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Alias(name, _) => name.clone(),
            Self::Container(name, ty) => format!("{}<{}>", name, ty.name()),
            Self::Custom(custom) => custom.rs_ty.clone(),
            Self::Enum(name, generic_args, _, _) => format_name_with_types(name, generic_args),
            Self::GenericArgument(arg) => arg.name.clone(),
            Self::List(name, ty) => format!("{}<{}>", name, ty.name()),
            Self::Map(name, key, value) => format!("{}<{}, {}>", name, key.name(), value.name()),
            Self::Primitive(primitive) => primitive.name(),
            Self::String => "String".to_owned(),
            Self::Struct(name, generic_args, _, _) => format_name_with_types(name, generic_args),
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustomType {
    pub name: String,
    pub type_args: Vec<Type>,
    pub rs_ty: String,
    pub ts_ty: String,
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

fn format_name_with_types(name: &str, generic_args: &[GenericArgument]) -> String {
    if generic_args.is_empty() {
        name.to_owned()
    } else {
        format!(
            "{}<{}>",
            name,
            generic_args
                .iter()
                .map(|arg| match &arg.ty {
                    Some(ty) => ty.name(),
                    None => arg.name.clone(),
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
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
                        Type::Alias(name, _) => name == &path_without_args && type_args.is_empty(),
                        Type::Container(name, ty) => {
                            name == &path_without_args
                                && type_args.len() == 1
                                && type_args
                                    .first()
                                    .map(|arg| arg == ty.as_ref())
                                    .unwrap_or(false)
                        }
                        Type::Custom(custom) => {
                            custom.name == path_without_args && custom.type_args == type_args
                        }
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
                        Type::Primitive(primitive) => primitive.name() == path_without_args,
                        Type::String => path_without_args == "String",
                        Type::Struct(name, generic_args, _, _) => {
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
