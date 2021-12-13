#[cfg(feature = "http-compat")]
mod http;
#[cfg(feature = "serde-bytes-compat")]
mod serde_bytes;
#[cfg(feature = "time-compat")]
mod time;

use crate::{
    generics::{contains_generic_arg, specialize_type_with_dependencies},
    types::{EnumOptions, GenericArgument, Variant, VariantAttrs},
    Type,
};
use dashmap::{mapref::one::Ref, DashMap};
use once_cell::sync::Lazy;
use std::{
    any::TypeId,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    rc::Rc,
};

#[cfg(feature = "serde-bytes-compat")]
mod serde_bytes;
#[cfg(feature = "time-compat")]
mod time;

pub trait Serializable: 'static {
    /// The name of the type as defined in the protocol.
    fn name() -> String;

    /// The type definition.
    fn ty() -> Type;

    /// Whether this type is a primitive.
    fn is_primitive() -> bool {
        false
    }

    /// Builder function for the `dependencies()` function. Trait
    /// implementations can implement this function, while using the default
    /// implementation for `dependencies()` to make sure the dependencies are
    /// only built and resolved once.
    fn build_dependencies() -> BTreeSet<Type>;

    /// Other (non-primitive) data structures this data structure depends on.
    fn dependencies() -> Ref<'static, TypeId, BTreeSet<Type>> {
        let type_id = TypeId::of::<Self>();
        static CACHED_DEPENDENCIES: Lazy<DashMap<TypeId, BTreeSet<Type>>> = Lazy::new(DashMap::new);

        if !CACHED_DEPENDENCIES.contains_key(&type_id) {
            CACHED_DEPENDENCIES.insert(type_id, Self::build_dependencies());
        }

        CACHED_DEPENDENCIES.get(&type_id).unwrap()
    }

    /// Returns a set with the type of `Self` and all of its dependencies.
    fn type_with_dependencies() -> BTreeSet<Type> {
        Self::named_type_with_dependencies("")
    }

    /// Returns a set with the type of `Self`, instantiated using the given `name`,
    /// and all of its dependencies.
    fn named_type_with_dependencies(name: &str) -> BTreeSet<Type> {
        Self::named_type_with_dependencies_and_generics(name, "")
    }

    /// Returns a set with the type of `Self` and all of its dependencies, but specializes
    /// generic arguments with specialized types.
    ///
    /// In addition, we receive the declaration of the generic arguments of the type
    /// whose `dependencies` we are gathering. This helps us to determine generic
    /// arguments and/or aliases in the given `name`.
    ///
    /// ## Example:
    ///
    /// If `Self` refers to `Option<T>`, but `name` is `Option<f64>`, we can specialize
    /// the `T` argument of the `Option` to be `f64` instead.
    fn named_type_with_dependencies_and_generics(name: &str, generic_args: &str) -> BTreeSet<Type> {
        let mut dependencies = if Self::is_primitive() {
            BTreeSet::new()
        } else {
            specialize_type_with_dependencies(Self::ty(), name, &Self::dependencies())
        };

        if !name.is_empty() && Self::name() != name {
            let generic_args = if generic_args.is_empty() {
                syn::Generics::default()
            } else {
                syn::parse_str(generic_args).expect("Cannot parse generic arguments")
            };
            let generic_arg = generic_args
                .params
                .iter()
                .find_map(|param| match param {
                    syn::GenericParam::Type(generic_ty) if generic_ty.ident == name => {
                        Some(generic_ty)
                    }
                    _ => None,
                })
                .map(|generic_ty| GenericArgument {
                    name: generic_ty.ident.to_string(),
                    ty: None,
                });

            if let Some(dependency) = match generic_arg {
                Some(arg) => Some(Type::GenericArgument(Box::new(arg))),
                None if !contains_generic_arg(name) => {
                    Some(Type::Alias(name.to_owned(), Box::new(Self::ty())))
                }
                _ => None,
            } {
                dependencies.insert(dependency);
            }
        }

        dependencies
    }
}

impl<T> Serializable for Box<T>
where
    T: Serializable,
{
    fn name() -> String {
        format!("Box<{}>", T::name())
    }

    fn ty() -> Type {
        Type::Container("Box".to_owned(), Box::new(T::ty()))
    }

    fn build_dependencies() -> BTreeSet<Type> {
        T::type_with_dependencies()
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

    fn build_dependencies() -> BTreeSet<Type> {
        let mut dependencies = BTreeSet::new();
        dependencies.append(&mut K::type_with_dependencies());
        dependencies.append(&mut V::type_with_dependencies());
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

    fn build_dependencies() -> BTreeSet<Type> {
        T::type_with_dependencies()
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

    fn build_dependencies() -> BTreeSet<Type> {
        let mut dependencies = BTreeSet::new();
        dependencies.append(&mut K::type_with_dependencies());
        dependencies.append(&mut V::type_with_dependencies());
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

    fn build_dependencies() -> BTreeSet<Type> {
        T::type_with_dependencies()
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
        Type::Container("Option".to_owned(), Box::new(T::ty()))
    }

    fn build_dependencies() -> BTreeSet<Type> {
        T::type_with_dependencies()
    }
}

impl<T> Serializable for Rc<T>
where
    T: Serializable,
{
    fn name() -> String {
        format!("Rc<{}>", T::name())
    }

    fn ty() -> Type {
        Type::Container("Rc".to_owned(), Box::new(T::ty()))
    }

    fn build_dependencies() -> BTreeSet<Type> {
        T::type_with_dependencies()
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
                    ty: None,
                },
                GenericArgument {
                    name: "E".to_owned(),
                    ty: None,
                },
            ],
            vec![
                " A result that can be either successful (`Ok)` or represent an error (`Err`)."
                    .to_owned(),
            ],
            vec![
                Variant {
                    name: "Ok".to_owned(),
                    ty: Type::Tuple(vec![Type::named_generic("T")]),
                    doc_lines: vec![" Represents a succesful result.".to_owned()],
                    attrs: VariantAttrs::default(),
                },
                Variant {
                    name: "Err".to_owned(),
                    ty: Type::Tuple(vec![Type::named_generic("E")]),
                    doc_lines: vec![" Represents an error.".to_owned()],
                    attrs: VariantAttrs::default(),
                },
            ],
            EnumOptions::default(),
        )
    }

    fn build_dependencies() -> BTreeSet<Type> {
        let mut dependencies = BTreeSet::new();
        dependencies.append(&mut T::type_with_dependencies());
        dependencies.append(&mut E::type_with_dependencies());
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

    fn build_dependencies() -> BTreeSet<Type> {
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

    fn build_dependencies() -> BTreeSet<Type> {
        T::type_with_dependencies()
    }
}

#[cfg(test)]
mod tests {
    use super::Serializable;
    use crate::{
        casing::Casing,
        primitives::Primitive,
        types::{Field, FieldAttrs, GenericArgument, StructOptions},
        Type,
    };
    use pretty_assertions::assert_eq;
    use std::collections::{BTreeMap, BTreeSet};

    pub struct Point<T>
    where
        T: Serializable,
    {
        pub value: T,
    }

    // Reflects actual macro output:
    impl<T> Serializable for Point<T>
    where
        T: Serializable,
    {
        fn name() -> String {
            Self::ty().name()
        }
        fn ty() -> Type {
            Type::from_item(
                "pub struct Point < T > {pub value : T ,}",
                &Self::dependencies(),
            )
        }
        fn build_dependencies() -> BTreeSet<Type> {
            let generics = "< T >";
            T::named_type_with_dependencies_and_generics("T", generics)
        }
    }

    #[test]
    pub fn test_point_dependencies() {
        let mut expected_dependencies = BTreeSet::new();
        expected_dependencies.insert(Type::GenericArgument(Box::new(GenericArgument {
            name: "T".to_owned(),
            ty: None,
        })));

        assert_eq!(Point::<f64>::build_dependencies(), expected_dependencies);
    }

    pub struct Complex {
        pub points: Vec<Point<f64>>,
    }

    impl Serializable for Complex {
        fn name() -> String {
            "Complex".to_owned()
        }
        fn ty() -> Type {
            Type::from_item(
                "pub struct Complex {pub points : Vec < Point < f64 >>,}",
                &Self::dependencies(),
            )
        }
        fn build_dependencies() -> BTreeSet<Type> {
            let generics = "";
            Vec::<Point<f64>>::named_type_with_dependencies_and_generics(
                "Vec<Point<f64>>",
                generics,
            )
        }
    }

    #[test]
    pub fn test_complex_dependencies() {
        let point = Type::Struct(
            "Point".to_owned(),
            vec![GenericArgument {
                name: "T".to_owned(),
                ty: Some(Type::Primitive(Primitive::F64)),
            }],
            vec![],
            vec![Field {
                name: "value".to_owned(),
                doc_lines: vec![],
                ty: Type::GenericArgument(Box::new(GenericArgument {
                    name: "T".to_owned(),
                    ty: Some(Type::Primitive(Primitive::F64)),
                })),
                attrs: FieldAttrs::default(),
            }],
            StructOptions {
                field_casing: Casing::CamelCase,
                native_modules: BTreeMap::new(),
            },
        );

        let mut expected_dependencies = BTreeSet::new();
        expected_dependencies.insert(point.clone());
        expected_dependencies.insert(Type::List("Vec".to_owned(), Box::new(point)));

        assert_eq!(Complex::build_dependencies(), expected_dependencies);
    }

    pub struct ComplexNested {
        pub complex_nested: BTreeMap<String, Vec<Point<f64>>>,
    }

    // Reflects actual macro output:
    impl Serializable for ComplexNested {
        fn name() -> String {
            "ComplexNested".to_owned()
        }
        fn ty() -> Type {
            Type::from_item("pub struct ComplexNested {pub complex_nested : BTreeMap < String , Vec < Point < f64 >>>,}", &Self::dependencies())
        }
        fn build_dependencies() -> BTreeSet<Type> {
            let generics = "";
            BTreeMap::<String, Vec<Point<f64>>>::named_type_with_dependencies_and_generics(
                "BTreeMap<String, Vec<Point<f64>>>",
                generics,
            )
        }
    }

    #[test]
    pub fn test_complex_nested_dependencies() {
        let specialized_argument = GenericArgument {
            name: "T".to_owned(),
            ty: Some(Type::Primitive(Primitive::F64)),
        };

        let point = Type::Struct(
            "Point".to_owned(),
            vec![GenericArgument {
                name: "T".to_owned(),
                ty: None,
            }],
            vec![],
            vec![Field {
                name: "value".to_owned(),
                doc_lines: vec![],
                ty: Type::GenericArgument(Box::new(GenericArgument {
                    name: "T".to_owned(),
                    ty: None,
                })),
                attrs: FieldAttrs::default(),
            }],
            StructOptions {
                field_casing: Casing::CamelCase,
                native_modules: BTreeMap::new(),
            },
        );
        let vec = Type::List("Vec".to_owned(), Box::new(point.clone()));
        let map = Type::Map(
            "BTreeMap".to_owned(),
            Box::new(Type::String),
            Box::new(
                vec.clone()
                    .with_specialized_args(&[specialized_argument.clone()]),
            ),
        );

        let mut expected_dependencies = BTreeSet::new();
        expected_dependencies.insert(
            point
                .clone()
                .with_specialized_args(&[specialized_argument.clone()]),
        );
        expected_dependencies.insert(vec.clone().with_specialized_args(&[specialized_argument]));
        expected_dependencies.insert(map);
        expected_dependencies.insert(Type::String);
        // FIXME: Should these really be necessary to be inserted?
        expected_dependencies.insert(point);
        expected_dependencies.insert(vec);
        expected_dependencies.insert(Type::GenericArgument(Box::new(GenericArgument {
            name: "T".to_owned(),
            ty: None,
        })));

        assert_eq!(ComplexNested::build_dependencies(), expected_dependencies);
    }

    pub struct Recursive {
        pub recursive: Point<Point<f64>>,
    }

    // Reflects actual macro output:
    impl Serializable for Recursive {
        fn name() -> String {
            "Recursive".to_owned()
        }
        fn ty() -> Type {
            Type::from_item(
                "pub struct Recursive {pub recursive : Point < Point < f64 >>,}",
                &Self::dependencies(),
            )
        }
        fn build_dependencies() -> BTreeSet<Type> {
            let generics = "";
            Point::<Point<f64>>::named_type_with_dependencies_and_generics(
                "Point<Point<f64>>",
                generics,
            )
        }
    }

    #[test]
    pub fn test_recursive_dependencies() {
        let specialized_argument = GenericArgument {
            name: "T".to_owned(),
            ty: Some(Type::Primitive(Primitive::F64)),
        };
        let point = Type::Struct(
            "Point".to_owned(),
            vec![GenericArgument {
                name: "T".to_owned(),
                ty: None,
            }],
            vec![],
            vec![Field {
                name: "value".to_owned(),
                doc_lines: vec![],
                ty: Type::GenericArgument(Box::new(GenericArgument {
                    name: "T".to_owned(),
                    ty: None,
                })),
                attrs: FieldAttrs::default(),
            }],
            StructOptions {
                field_casing: Casing::CamelCase,
                native_modules: BTreeMap::new(),
            },
        );
        let specialized_point = point.clone().with_specialized_args(&[specialized_argument]);
        let point_point = point.with_specialized_args(&[GenericArgument {
            name: "T".to_owned(),
            ty: Some(specialized_point.clone()),
        }]);

        let mut expected_dependencies = BTreeSet::new();
        expected_dependencies.insert(specialized_point);
        expected_dependencies.insert(point_point);

        assert_eq!(Recursive::build_dependencies(), expected_dependencies);
    }

    pub struct NestedRecursive {
        pub nested_recursive: Vec<Point<Point<f64>>>,
    }

    // Reflects actual macro output:
    impl Serializable for NestedRecursive {
        fn name() -> String {
            "NestedRecursive".to_owned()
        }
        fn ty() -> Type {
            Type::from_item(
                "pub struct NestedRecursive {pub nested_recursive : Vec < Point < Point < f64 >>>,}",
                &Self::dependencies(),
            )
        }
        fn build_dependencies() -> BTreeSet<Type> {
            let generics = "";
            Vec::<Point<Point<f64>>>::named_type_with_dependencies_and_generics(
                "Vec<Point<Point<f64>>>",
                generics,
            )
        }
    }

    #[test]
    pub fn test_nested_recursive_dependencies() {
        let specialized_argument = GenericArgument {
            name: "T".to_owned(),
            ty: Some(Type::Primitive(Primitive::F64)),
        };
        let point = Type::Struct(
            "Point".to_owned(),
            vec![GenericArgument {
                name: "T".to_owned(),
                ty: None,
            }],
            vec![],
            vec![Field {
                name: "value".to_owned(),
                doc_lines: vec![],
                ty: Type::GenericArgument(Box::new(GenericArgument {
                    name: "T".to_owned(),
                    ty: None,
                })),
                attrs: FieldAttrs::default(),
            }],
            StructOptions {
                field_casing: Casing::CamelCase,
                native_modules: BTreeMap::new(),
            },
        );
        let specialized_point = point.clone().with_specialized_args(&[specialized_argument]);
        let point_point = point.with_specialized_args(&[GenericArgument {
            name: "T".to_owned(),
            ty: Some(specialized_point.clone()),
        }]);
        let vec = Type::List("Vec".to_owned(), Box::new(point_point.clone()));

        let mut expected_dependencies = BTreeSet::new();
        expected_dependencies.insert(specialized_point);
        expected_dependencies.insert(point_point);
        expected_dependencies.insert(vec);

        assert_eq!(NestedRecursive::build_dependencies(), expected_dependencies);
    }

    pub struct CombinedFields {
        pub list: Vec<f64>,
        pub points: Vec<Point<f64>>,
        pub nested_recursive: Vec<Point<Point<f64>>>,
    }

    // Reflects actual macro output:
    impl Serializable for CombinedFields {
        fn name() -> String {
            "CombinedFields".to_owned()
        }
        fn ty() -> Type {
            Type::from_item(
                "pub struct CombinedFields {pub points : Vec < Point < f64 > >,pub nested_recursive : Vec < Point < Point < f64 >>>,}",
                &Self::dependencies(),
            )
        }
        fn build_dependencies() -> BTreeSet<Type> {
            let generics = "";
            let mut dependencies = BTreeSet::new();
            dependencies.append(&mut Vec::<f64>::named_type_with_dependencies_and_generics(
                "Vec<f64>", generics,
            ));
            dependencies.append(
                &mut Vec::<Point<f64>>::named_type_with_dependencies_and_generics(
                    "Vec<Point<f64>>",
                    generics,
                ),
            );
            dependencies.append(
                &mut Vec::<Point<Point<f64>>>::named_type_with_dependencies_and_generics(
                    "Vec<Point<Point<f64>>>",
                    generics,
                ),
            );
            dependencies
        }
    }

    #[test]
    pub fn test_combined_fields_dependencies() {
        let specialized_argument = GenericArgument {
            name: "T".to_owned(),
            ty: Some(Type::Primitive(Primitive::F64)),
        };
        let point = Type::Struct(
            "Point".to_owned(),
            vec![GenericArgument {
                name: "T".to_owned(),
                ty: None,
            }],
            vec![],
            vec![Field {
                name: "value".to_owned(),
                doc_lines: vec![],
                ty: Type::GenericArgument(Box::new(GenericArgument {
                    name: "T".to_owned(),
                    ty: None,
                })),
                attrs: FieldAttrs::default(),
            }],
            StructOptions {
                field_casing: Casing::CamelCase,
                native_modules: BTreeMap::new(),
            },
        );
        let specialized_point = point.clone().with_specialized_args(&[specialized_argument]);
        let point_point = point.with_specialized_args(&[GenericArgument {
            name: "T".to_owned(),
            ty: Some(specialized_point.clone()),
        }]);
        let vec_f64 = Type::List("Vec".to_owned(), Box::new(Type::Primitive(Primitive::F64)));
        let vec_point = Type::List("Vec".to_owned(), Box::new(specialized_point.clone()));
        let vec_point_point = Type::List("Vec".to_owned(), Box::new(point_point.clone()));

        let mut expected_dependencies = BTreeSet::new();
        expected_dependencies.insert(specialized_point);
        expected_dependencies.insert(point_point);
        expected_dependencies.insert(vec_f64);
        expected_dependencies.insert(vec_point);
        expected_dependencies.insert(vec_point_point);

        assert_eq!(CombinedFields::build_dependencies(), expected_dependencies);
    }
}
