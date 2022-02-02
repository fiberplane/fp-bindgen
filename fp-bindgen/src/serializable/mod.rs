use crate::{
    types::{Enum, EnumOptions, TypeIdent, Variant, VariantAttrs},
    Type,
};
use once_cell::sync::Lazy;
use std::{
    any::TypeId,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    rc::Rc,
    sync::RwLock,
};

#[cfg(feature = "http-compat")]
mod http;
#[cfg(feature = "serde-bytes-compat")]
mod serde_bytes;
#[cfg(feature = "time-compat")]
mod time;

static CACHED_TYPES: Lazy<RwLock<HashMap<TypeId, Type>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
static CACHED_DEPENDENCIES: Lazy<RwLock<HashMap<TypeId, BTreeSet<Type>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub trait Serializable: 'static {
    /// The identifier of the type as defined in the protocol.
    fn ident() -> TypeIdent;

    /// The type definition.
    fn ty() -> Type;

    /// Whether this type is a primitive.
    fn is_primitive() -> bool {
        false
    }

    /// Collects all the identifiers of the type and its dependencies.
    ///
    /// The default implementation is only suitable for types without
    /// dependencies.
    fn collect_dependency_idents(idents: &mut BTreeSet<TypeIdent>) {
        idents.insert(Self::ident());
    }
}

impl<T> Serializable for Box<T>
where
    T: Serializable,
{
    fn ident() -> TypeIdent {
        TypeIdent {
            name: "Box".to_owned(),
            generic_args: vec![TypeIdent::from("T")],
        }
    }

    fn ty() -> Type {
        Type::Container("Box".to_owned(), TypeIdent::from("T"))
    }

    fn collect_dependency_idents(idents: &mut BTreeSet<TypeIdent>) {
        if idents.insert(Self::ident()) {
            T::collect_dependency_idents(idents);
        }
    }
}

impl<K, V> Serializable for BTreeMap<K, V>
where
    K: Serializable,
    V: Serializable,
{
    fn ident() -> TypeIdent {
        TypeIdent {
            name: "BTreeMap".to_owned(),
            generic_args: vec![TypeIdent::from("K"), TypeIdent::from("V")],
        }
    }

    fn ty() -> Type {
        Type::Map(
            "BTreeMap".to_owned(),
            TypeIdent::from("K"),
            TypeIdent::from("V"),
        )
    }

    fn collect_dependency_idents(idents: &mut BTreeSet<TypeIdent>) {
        if idents.insert(Self::ident()) {
            K::collect_dependency_idents(idents);
            V::collect_dependency_idents(idents);
        }
    }
}

impl<T> Serializable for BTreeSet<T>
where
    T: Serializable,
{
    fn ident() -> TypeIdent {
        TypeIdent {
            name: "BTreeSet".to_owned(),
            generic_args: vec![TypeIdent::from("T")],
        }
    }

    fn ty() -> Type {
        Type::List("BTreeSet".to_owned(), TypeIdent::from("T"))
    }

    fn collect_dependency_idents(idents: &mut BTreeSet<TypeIdent>) {
        if idents.insert(Self::ident()) {
            T::collect_dependency_idents(idents);
        }
    }
}

impl<K, V> Serializable for HashMap<K, V>
where
    K: Serializable,
    V: Serializable,
{
    fn ident() -> TypeIdent {
        TypeIdent {
            name: "HashMap".to_owned(),
            generic_args: vec![TypeIdent::from("K"), TypeIdent::from("V")],
        }
    }

    fn ty() -> Type {
        Type::Map(
            "HashMap".to_owned(),
            TypeIdent::from("K"),
            TypeIdent::from("V"),
        )
    }

    fn collect_dependency_idents(idents: &mut BTreeSet<TypeIdent>) {
        if idents.insert(Self::ident()) {
            K::collect_dependency_idents(idents);
            V::collect_dependency_idents(idents);
        }
    }
}

impl<T> Serializable for HashSet<T>
where
    T: Serializable,
{
    fn ident() -> TypeIdent {
        TypeIdent {
            name: "HashSet".to_owned(),
            generic_args: vec![TypeIdent::from("T")],
        }
    }

    fn ty() -> Type {
        Type::List("HashSet".to_owned(), TypeIdent::from("T"))
    }

    fn collect_dependency_idents(idents: &mut BTreeSet<TypeIdent>) {
        if idents.insert(Self::ident()) {
            T::collect_dependency_idents(idents);
        }
    }
}

impl<T> Serializable for Option<T>
where
    T: Serializable,
{
    fn ident() -> TypeIdent {
        TypeIdent {
            name: "Option".to_owned(),
            generic_args: vec![TypeIdent::from("T")],
        }
    }

    fn ty() -> Type {
        Type::Container("Option".to_owned(), TypeIdent::from("T"))
    }

    fn collect_dependency_idents(idents: &mut BTreeSet<TypeIdent>) {
        if idents.insert(Self::ident()) {
            T::collect_dependency_idents(idents);
        }
    }
}

impl<T> Serializable for Rc<T>
where
    T: Serializable,
{
    fn ident() -> TypeIdent {
        TypeIdent {
            name: "Rc".to_owned(),
            generic_args: vec![TypeIdent::from("T")],
        }
    }

    fn ty() -> Type {
        Type::Container("Rc".to_owned(), TypeIdent::from("T"))
    }

    fn collect_dependency_idents(idents: &mut BTreeSet<TypeIdent>) {
        if idents.insert(Self::ident()) {
            T::collect_dependency_idents(idents);
        }
    }
}

impl<T, E> Serializable for Result<T, E>
where
    T: Serializable,
    E: Serializable,
{
    fn ident() -> TypeIdent {
        TypeIdent {
            name: "Result".to_owned(),
            generic_args: vec![TypeIdent::from("T"), TypeIdent::from("E")],
        }
    }

    fn ty() -> Type {
        Type::Enum(Enum {
            ident: Self::ident(),
            variants: vec![
                Variant {
                    name: "Ok".to_owned(),
                    ty: Type::Tuple(vec![TypeIdent::from("T")]),
                    doc_lines: vec![" Represents a succesful result.".to_owned()],
                    attrs: VariantAttrs::default(),
                },
                Variant {
                    name: "Err".to_owned(),
                    ty: Type::Tuple(vec![TypeIdent::from("E")]),
                    doc_lines: vec![" Represents an error.".to_owned()],
                    attrs: VariantAttrs::default(),
                },
            ],
            doc_lines: vec![
                " A result that can be either successful (`Ok)` or represent an error (`Err`)."
                    .to_owned(),
            ],
            options: EnumOptions::default(),
        })
    }

    fn collect_dependency_idents(idents: &mut BTreeSet<TypeIdent>) {
        if idents.insert(Self::ident()) {
            T::collect_dependency_idents(idents);
            E::collect_dependency_idents(idents);
        }
    }
}

impl Serializable for String {
    fn ident() -> TypeIdent {
        TypeIdent::from("String")
    }

    fn ty() -> Type {
        Type::String
    }
}

impl<T> Serializable for Vec<T>
where
    T: Serializable,
{
    fn ident() -> TypeIdent {
        TypeIdent {
            name: "Vec".to_owned(),
            generic_args: vec![TypeIdent::from("T")],
        }
    }

    fn ty() -> Type {
        Type::List("Vec".to_owned(), TypeIdent::from("T"))
    }

    fn collect_dependency_idents(idents: &mut BTreeSet<TypeIdent>) {
        if idents.insert(Self::ident()) {
            T::collect_dependency_idents(idents);
        }
    }
}
