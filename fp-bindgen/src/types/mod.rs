use crate::primitives::Primitive;
use quote::ToTokens;
use std::{
    collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet},
    hash::{Hash, Hasher},
    str::FromStr,
};
use syn::{Item, PathArguments};

mod enums;
mod structs;

pub use enums::{EnumOptions, Variant};
pub use structs::{Field, FieldAttrs, StructOptions};

/// A generic argument has a name (T, E, ...) and an optional type, which is only known in contexts
/// when we are dealing with concrete instances of the generic type.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GenericArgument {
    pub name: String,
    pub ty: Option<Type>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    Alias(String, Box<Type>),
    Container(String, Box<Type>),
    Custom(CustomType),
    Enum(
        String,
        Vec<GenericArgument>,
        Vec<String>,
        Vec<Variant>,
        EnumOptions,
    ),
    GenericArgument(Box<GenericArgument>),
    List(String, Box<Type>),
    Map(String, Box<Type>, Box<Type>),
    Primitive(Primitive),
    String,
    Struct(
        String,
        Vec<GenericArgument>,
        Vec<String>,
        Vec<Field>,
        StructOptions,
    ),
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

    pub fn generic_args(&self) -> Vec<GenericArgument> {
        match self {
            Self::Enum(_, generic_args, _, _, _) => generic_args.clone(),
            Self::Struct(_, generic_args, _, _, _) => generic_args.clone(),
            _ => vec![],
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Alias(name, _) => name.clone(),
            Self::Container(name, ty) => format!("{}<{}>", name, ty.name()),
            Self::Custom(custom) => custom.rs_ty.clone(),
            Self::Enum(name, generic_args, _, _, _) => format_name_with_types(name, generic_args),
            Self::GenericArgument(arg) => arg.name.clone(),
            Self::List(name, ty) => format!("{}<{}>", name, ty.name()),
            Self::Map(name, key, value) => format!("{}<{}, {}>", name, key.name(), value.name()),
            Self::Primitive(primitive) => primitive.name(),
            Self::String => "String".to_owned(),
            Self::Struct(name, generic_args, _, _, _) => format_name_with_types(name, generic_args),
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

    fn ord_key(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        format!("{}@{}", self.name(), hasher.finish())
    }

    pub fn with_specialized_args(self, specialized_args: &[GenericArgument]) -> Self {
        let specialize_arg = |arg: GenericArgument| {
            let name = arg.name;
            let ty = specialized_args
                .iter()
                .find(|specialized| specialized.name == name)
                .and_then(|specialized| specialized.ty.clone())
                .or(arg.ty);
            GenericArgument { name, ty }
        };
        let specialize_args =
            |args: Vec<GenericArgument>| args.into_iter().map(specialize_arg).collect();

        match self {
            Self::Container(name, item) => {
                Self::Container(name, Box::new(item.with_specialized_args(specialized_args)))
            }
            Self::Map(name, key, value) => Self::Map(
                name,
                key,
                Box::new(value.with_specialized_args(specialized_args)),
            ),
            Self::Enum(name, args, doc_lines, variants, opts) => Self::Enum(
                name,
                specialize_args(args),
                doc_lines,
                variants
                    .into_iter()
                    .map(|variant| Variant {
                        doc_lines: variant.doc_lines,
                        name: variant.name,
                        ty: variant.ty.with_specialized_args(specialized_args),
                    })
                    .collect(),
                opts,
            ),
            Self::GenericArgument(arg) => Self::GenericArgument(Box::new(specialize_arg(*arg))),
            Self::List(name, item) => {
                Self::List(name, Box::new(item.with_specialized_args(specialized_args)))
            }
            Self::Struct(name, args, doc_lines, fields, opts) => Self::Struct(
                name,
                specialize_args(args),
                doc_lines,
                fields
                    .into_iter()
                    .map(|field| Field {
                        doc_lines: field.doc_lines,
                        name: field.name,
                        ty: field.ty.with_specialized_args(specialized_args),
                        attrs: field.attrs,
                    })
                    .collect(),
                opts,
            ),
            other => other,
        }
    }
}

impl Ord for Type {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ord_key().cmp(&other.ord_key())
    }
}

impl PartialOrd for Type {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.ord_key().partial_cmp(&other.ord_key())
    }
}

/// Used for defining type information for types that are defined externally,
/// or that otherwise require custom treatment.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CustomType {
    pub name: String,
    pub type_args: Vec<Type>,

    /// Qualified path to refer to the type in the Rust generators.
    pub rs_ty: String,

    /// Dependencies to add to the Rust plugin's `Cargo.toml` to be able to
    /// use the type.
    ///
    /// Keys in the map are dependency names as they appear on the left-hand
    /// side of the `=` in the `Cargo.toml` `[dependencies]` section, while the
    /// value is the literal part that comes on the right-hand side.
    pub rs_dependencies: BTreeMap<String, String>,

    /// Name to refer to the type in the TypeScript generator.
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
                        Type::Enum(name, generic_args, _, _, _) => {
                            name == &path_without_args
                                && generic_args_match_type_args(generic_args, &type_args)
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
                        Type::Struct(name, generic_args, _, _, _) => {
                            name == &path_without_args
                                && generic_args_match_type_args(generic_args, &type_args)
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
                .map(|ty| resolve_type_or_panic(ty, types, "Unresolvable type in tuple"))
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

/// As `resolve_type()`, but panics when resolving the type fails.
pub fn resolve_type_or_panic(ty: &syn::Type, types: &BTreeSet<Type>, error_message: &str) -> Type {
    resolve_type(ty, types).unwrap_or_else(|| {
        panic!(
            "{}: {:?}\ndependencies:\n{}",
            error_message,
            ty.to_token_stream().to_string(),
            types
                .iter()
                .map(|ty| format!("  {}", ty.name()))
                .collect::<Vec<_>>()
                .join("\n")
        );
    })
}

fn generic_args_match_type_args(generic_args: &[GenericArgument], type_args: &[Type]) -> bool {
    if generic_args.len() != type_args.len() {
        return false;
    }

    generic_args
        .iter()
        .zip(type_args.iter())
        .all(|(generic_arg, type_arg)| {
            generic_arg.name == type_arg.name()
                || match type_arg {
                    Type::GenericArgument(ty_arg) if generic_arg.ty.is_some() => {
                        generic_arg.ty == ty_arg.ty
                    }
                    ty => generic_arg
                        .ty
                        .as_ref()
                        .map(|generic_ty| generic_ty == ty)
                        .unwrap_or(false),
                }
        })
}

#[cfg(test)]
mod tests {
    use super::resolve_type;
    use crate::{
        primitives::Primitive,
        types::{GenericArgument, StructOptions},
        Type,
    };
    use std::collections::BTreeSet;
    use syn::parse_quote;

    #[test]
    fn test_resolve_generic_type() {
        let ty: syn::Type = parse_quote!(Vec<Point<T>>);

        let t = Type::GenericArgument(Box::new(GenericArgument {
            name: "T".to_owned(),
            ty: Some(Type::Primitive(Primitive::F64)),
        }));
        let point = Type::Struct(
            "Point".to_owned(),
            vec![GenericArgument {
                name: "T".to_owned(),
                ty: None,
            }],
            vec![],
            vec![],
            StructOptions::default(),
        );
        let vec = Type::List("Vec".to_owned(), Box::new(point.clone()));

        let mut types = BTreeSet::new();
        types.insert(t);
        types.insert(point);
        types.insert(vec.clone());

        assert_eq!(resolve_type(&ty, &types), Some(vec));
    }

    #[test]
    fn test_resolve_specialized_type() {
        let ty: syn::Type = parse_quote!(Point<f64>);

        let t = Type::GenericArgument(Box::new(GenericArgument {
            name: "T".to_owned(),
            ty: Some(Type::Primitive(Primitive::F64)),
        }));
        let point = Type::Struct(
            "Point".to_owned(),
            vec![GenericArgument {
                name: "T".to_owned(),
                ty: Some(Type::Primitive(Primitive::F64)),
            }],
            vec![],
            vec![],
            StructOptions::default(),
        );

        let mut types = BTreeSet::new();
        types.insert(t);
        types.insert(point.clone());

        assert_eq!(resolve_type(&ty, &types), Some(point));
    }

    #[test]
    fn test_resolve_nested_specialized_type() {
        let ty: syn::Type = parse_quote!(Vec<Point<f64>>);

        let t = Type::GenericArgument(Box::new(GenericArgument {
            name: "T".to_owned(),
            ty: Some(Type::Primitive(Primitive::F64)),
        }));
        let point = Type::Struct(
            "Point".to_owned(),
            vec![GenericArgument {
                name: "T".to_owned(),
                ty: Some(Type::Primitive(Primitive::F64)),
            }],
            vec![],
            vec![],
            StructOptions::default(),
        );
        let vec = Type::List("Vec".to_owned(), Box::new(point.clone()));

        let mut types = BTreeSet::new();
        types.insert(t);
        types.insert(point);
        types.insert(vec.clone());

        assert_eq!(resolve_type(&ty, &types), Some(vec));
    }
}
