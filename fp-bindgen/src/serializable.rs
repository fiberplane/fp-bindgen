use crate::{
    types::{GenericArgument, Variant},
    Type,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

pub trait Serializable {
    /// The name of the type as defined in the protocol.
    fn name() -> String;

    /// The type definition.
    fn ty() -> Type;

    /// Whether this type is a primitive.
    fn is_primitive() -> bool {
        false
    }

    /// Other (non-primitive) data structures this data structure depends on.
    fn dependencies() -> BTreeSet<Type>;

    fn add_type_with_dependencies(dependencies: &mut BTreeSet<Type>) {
        if Self::is_primitive() {
            return;
        }

        let ty = Self::ty();
        if dependencies.contains(&ty) {
            return;
        }

        dependencies.insert(ty);
        dependencies.append(&mut Self::dependencies());
    }
}

impl<K, V> Serializable for BTreeMap<K, V>
where
    K: Serializable,
    V: Serializable,
{
    fn name() -> String {
        format!("BTreeMap<{}, {}>", K::name(), V::name())
    }

    fn ty() -> Type {
        Type::Map("BTreeMap".to_owned(), Box::new(K::ty()), Box::new(V::ty()))
    }

    fn dependencies() -> BTreeSet<Type> {
        let mut dependencies = BTreeSet::new();
        K::add_type_with_dependencies(&mut dependencies);
        V::add_type_with_dependencies(&mut dependencies);
        dependencies
    }
}

impl<T> Serializable for BTreeSet<T>
where
    T: Serializable,
{
    fn name() -> String {
        format!("BTreeSet<{}>", T::name())
    }

    fn ty() -> Type {
        Type::List("BTreeSet".to_owned(), Box::new(T::ty()))
    }

    fn dependencies() -> BTreeSet<Type> {
        let mut dependencies = BTreeSet::new();
        T::add_type_with_dependencies(&mut dependencies);
        dependencies
    }
}

impl<K, V> Serializable for HashMap<K, V>
where
    K: Serializable,
    V: Serializable,
{
    fn name() -> String {
        format!("HashMap<{}, {}>", K::name(), V::name())
    }

    fn ty() -> Type {
        Type::Map("HashMap".to_owned(), Box::new(K::ty()), Box::new(V::ty()))
    }

    fn dependencies() -> BTreeSet<Type> {
        let mut dependencies = BTreeSet::new();
        K::add_type_with_dependencies(&mut dependencies);
        V::add_type_with_dependencies(&mut dependencies);
        dependencies
    }
}

impl<T> Serializable for HashSet<T>
where
    T: Serializable,
{
    fn name() -> String {
        format!("HashSet<{}>", T::name())
    }

    fn ty() -> Type {
        Type::List("HashSet".to_owned(), Box::new(T::ty()))
    }

    fn dependencies() -> BTreeSet<Type> {
        let mut dependencies = BTreeSet::new();
        T::add_type_with_dependencies(&mut dependencies);
        dependencies
    }
}

impl<T> Serializable for Option<T>
where
    T: Serializable,
{
    fn name() -> String {
        format!("Option<{}>", T::name())
    }

    fn ty() -> Type {
        Type::Option(Box::new(T::ty()))
    }

    fn dependencies() -> BTreeSet<Type> {
        let mut dependencies = BTreeSet::new();
        T::add_type_with_dependencies(&mut dependencies);
        dependencies
    }
}

impl<T, E> Serializable for Result<T, E>
where
    T: Serializable,
    E: Serializable,
{
    fn name() -> String {
        format!("Result<{}, {}>", T::name(), E::name())
    }

    fn ty() -> Type {
        Type::Enum(
            "Result".to_owned(),
            vec![
                GenericArgument {
                    name: "T".to_owned(),
                    ty: Some(T::ty()),
                },
                GenericArgument {
                    name: "E".to_owned(),
                    ty: Some(E::ty()),
                },
            ],
            vec![
                Variant {
                    name: "Ok".to_owned(),
                    ty: Type::Tuple(vec![Type::named_generic("T")]),
                },
                Variant {
                    name: "Err".to_owned(),
                    ty: Type::Tuple(vec![Type::named_generic("E")]),
                },
            ],
        )
    }

    fn dependencies() -> BTreeSet<Type> {
        let mut dependencies = BTreeSet::new();
        T::add_type_with_dependencies(&mut dependencies);
        E::add_type_with_dependencies(&mut dependencies);
        dependencies
    }
}

impl Serializable for String {
    fn name() -> String {
        "String".to_owned()
    }

    fn ty() -> Type {
        Type::String
    }

    fn dependencies() -> BTreeSet<Type> {
        BTreeSet::new()
    }
}

impl<T> Serializable for Vec<T>
where
    T: Serializable,
{
    fn name() -> String {
        format!("Vec<{}>", T::name())
    }

    fn ty() -> Type {
        Type::List("Vec".to_owned(), Box::new(T::ty()))
    }

    fn dependencies() -> BTreeSet<Type> {
        let mut dependencies = BTreeSet::new();
        T::add_type_with_dependencies(&mut dependencies);
        dependencies
    }
}
