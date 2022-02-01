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

    /// Identifiers of other types this type depends on. Only direct
    /// dependencies need to be returned.
    fn dependencies() -> HashSet<TypeIdent>;

    /// Collects all the identifiers of the type and its dependencies.
    fn collect_dependency_idents(idents: &mut HashSet<TypeIdent>) {
        if idents.insert(Self::ident()) {
            idents.extend(&mut Self::dependencies().drain())
        }
    }
}

impl<T> Serializable for Box<T>
where
    T: Serializable,
{
    fn ident() -> TypeIdent {
        TypeIdent {
            name: "Box".to_owned(),
            generic_args: vec![T::ident()],
        }
    }

    fn ty() -> Type {
        Type::Container("Box".to_owned(), T::ident())
    }

    fn dependencies() -> HashSet<TypeIdent> {
        HashSet::from([T::ident()])
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
            generic_args: vec![K::ident(), V::ident()],
        }
    }

    fn ty() -> Type {
        Type::Map("BTreeMap".to_owned(), K::ident(), V::ident())
    }

    fn dependencies() -> HashSet<TypeIdent> {
        HashSet::from([K::ident(), V::ident()])
    }
}

impl<T> Serializable for BTreeSet<T>
where
    T: Serializable,
{
    fn ident() -> TypeIdent {
        TypeIdent {
            name: "BTreeSet".to_owned(),
            generic_args: vec![T::ident()],
        }
    }

    fn ty() -> Type {
        Type::List("BTreeSet".to_owned(), T::ident())
    }

    fn dependencies() -> HashSet<TypeIdent> {
        HashSet::from([T::ident()])
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
            generic_args: vec![K::ident(), V::ident()],
        }
    }

    fn ty() -> Type {
        Type::Map("HashMap".to_owned(), K::ident(), V::ident())
    }

    fn dependencies() -> HashSet<TypeIdent> {
        HashSet::from([K::ident(), V::ident()])
    }
}

impl<T> Serializable for HashSet<T>
where
    T: Serializable,
{
    fn ident() -> TypeIdent {
        TypeIdent {
            name: "HashSet".to_owned(),
            generic_args: vec![T::ident()],
        }
    }

    fn ty() -> Type {
        Type::List("HashSet".to_owned(), T::ident())
    }

    fn dependencies() -> HashSet<TypeIdent> {
        HashSet::from([T::ident()])
    }
}

impl<T> Serializable for Option<T>
where
    T: Serializable,
{
    fn ident() -> TypeIdent {
        TypeIdent {
            name: "Option".to_owned(),
            generic_args: vec![T::ident()],
        }
    }

    fn ty() -> Type {
        Type::Container("Option".to_owned(), T::ident())
    }

    fn dependencies() -> HashSet<TypeIdent> {
        HashSet::from([T::ident()])
    }
}

impl<T> Serializable for Rc<T>
where
    T: Serializable,
{
    fn ident() -> TypeIdent {
        TypeIdent {
            name: "Rc".to_owned(),
            generic_args: vec![T::ident()],
        }
    }

    fn ty() -> Type {
        Type::Container("Rc".to_owned(), T::ident())
    }

    fn dependencies() -> HashSet<TypeIdent> {
        T::ident_with_dependencies()
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
            generic_args: vec![T::ident(), E::ident()],
        }
    }

    fn ty() -> Type {
        Type::Enum(Enum {
            ident: Self::ident(),
            variants: vec![
                Variant {
                    name: "Ok".to_owned(),
                    ty: Type::Tuple(vec![T::ident()]),
                    doc_lines: vec![" Represents a succesful result.".to_owned()],
                    attrs: VariantAttrs::default(),
                },
                Variant {
                    name: "Err".to_owned(),
                    ty: Type::Tuple(vec![E::ident()]),
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

    fn dependencies() -> HashSet<TypeIdent> {
        let mut dependencies = HashSet::new();
        dependencies.extend(&mut T::ident_with_dependencies().drain());
        dependencies.extend(&mut E::ident_with_dependencies().drain());
        dependencies
    }
}

impl Serializable for String {
    fn name() -> String {
        "String".to_owned()
    }

    fn build_ty() -> Type {
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

    fn build_ty() -> Type {
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
        fn build_ty() -> Type {
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
        fn build_ty() -> Type {
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
        fn build_ty() -> Type {
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
        fn build_ty() -> Type {
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
        fn build_ty() -> Type {
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
        fn build_ty() -> Type {
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
