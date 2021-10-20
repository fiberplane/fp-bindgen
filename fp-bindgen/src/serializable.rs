#[cfg(feature = "chrono-compat")]
use crate::CustomType;
use crate::{
    generics::{contains_generic_arg, specialize_type_with_dependencies},
    types::{EnumOptions, GenericArgument, Variant},
    Type,
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    rc::Rc,
};

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

    /// Adds `Self` to the set of `dependencies`.
    ///
    /// Dependencies of `Self` are recursively added to the `dependencies` as well.
    fn add_type_with_dependencies(dependencies: &mut BTreeSet<Type>) {
        Self::add_named_type_with_dependencies(dependencies, "")
    }

    /// Adds `Self` with the given `name` to the set of `dependencies`.
    ///
    /// Dependencies of `Self` are recursively added to the `dependencies` as well.
    fn add_named_type_with_dependencies(dependencies: &mut BTreeSet<Type>, name: &str) {
        Self::add_named_type_with_dependencies_and_generics(dependencies, name, "")
    }

    /// Adds `Self` with the given `name` to the set of `dependencies`, but specializes
    /// generic arguments with specialized types.
    ///
    /// Dependencies of `Self` are recursively added to the `dependencies` as well.
    ///
    /// In addition, we receive the declaration of the generic arguments of the type
    /// whose `dependencies` we are gathering. This helps us to determine generic
    /// arguments and/or aliases in the given `name`.
    ///
    /// ## Example:
    ///
    /// If `Self` refers to `Option<T>`, but `name` is `Option<f64>`, we can specialize
    /// the `T` argument of the `Option` to be `f64` instead.
    fn add_named_type_with_dependencies_and_generics(
        dependencies: &mut BTreeSet<Type>,
        name: &str,
        generic_args: &str,
    ) {
        let ty = Self::ty();
        if !name.is_empty() && ty.name() != name {
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
                    Some(Type::Alias(name.to_owned(), Box::new(ty.clone())))
                }
                _ => None,
            } {
                dependencies.insert(dependency);
            }
        }

        if Self::is_primitive() || dependencies.contains(&ty) {
            return;
        }

        dependencies.append(&mut specialize_type_with_dependencies(
            ty,
            name,
            &Self::dependencies(),
        ));
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

    fn dependencies() -> BTreeSet<Type> {
        let mut dependencies = BTreeSet::new();
        T::add_type_with_dependencies(&mut dependencies);
        dependencies
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
        Type::Container("Option".to_owned(), Box::new(T::ty()))
    }

    fn dependencies() -> BTreeSet<Type> {
        let mut dependencies = BTreeSet::new();
        T::add_type_with_dependencies(&mut dependencies);
        dependencies
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
                },
                Variant {
                    name: "Err".to_owned(),
                    ty: Type::Tuple(vec![Type::named_generic("E")]),
                    doc_lines: vec![" Represents an error.".to_owned()],
                },
            ],
            EnumOptions::default(),
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

#[cfg(feature = "chrono-compat")]
impl Serializable for chrono::Utc {
    fn name() -> String {
        "chrono::Utc".to_owned()
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            name: "Utc".to_owned(),
            type_args: vec![],
            rs_ty: "chrono::Utc".to_owned(),
            ts_ty: "UNIMPLEMENTED".to_owned(), // *should* never appear in the generated output
        })
    }

    fn dependencies() -> BTreeSet<Type> {
        BTreeSet::new()
    }
}

#[cfg(feature = "chrono-compat")]
impl<T> Serializable for chrono::DateTime<T>
where
    T: chrono::TimeZone + Serializable,
{
    fn name() -> String {
        "chrono::DateTime<T>".to_owned()
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            name: "DateTime".to_owned(),
            type_args: vec![],
            rs_ty: format!("chrono::DateTime<{}>", T::name()),
            ts_ty: "string".to_owned(),
        })
    }

    fn dependencies() -> BTreeSet<Type> {
        let mut dependencies = BTreeSet::new();
        T::add_type_with_dependencies(&mut dependencies);
        dependencies
    }
}

#[cfg(test)]
mod tests {
    use super::Serializable;
    use crate::{
        casing::Casing,
        primitives::Primitive,
        types::{Field, GenericArgument, StructOptions},
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
        fn dependencies() -> BTreeSet<Type> {
            let generics = "< T >";
            let mut dependencies = BTreeSet::new();
            T::add_named_type_with_dependencies_and_generics(&mut dependencies, "T", generics);
            dependencies
        }
    }

    #[test]
    pub fn test_point_dependencies() {
        let mut expected_dependencies = BTreeSet::new();
        expected_dependencies.insert(Type::GenericArgument(Box::new(GenericArgument {
            name: "T".to_owned(),
            ty: None,
        })));

        assert_eq!(Point::<f64>::dependencies(), expected_dependencies);
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
        fn dependencies() -> BTreeSet<Type> {
            let generics = "";
            let mut dependencies = BTreeSet::new();
            Vec::<Point<f64>>::add_named_type_with_dependencies_and_generics(
                &mut dependencies,
                "Vec<Point<f64>>",
                generics,
            );
            dependencies
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
            }],
            StructOptions {
                field_casing: Casing::CamelCase,
                native_modules: BTreeMap::new(),
            },
        );

        let mut expected_dependencies = BTreeSet::new();
        expected_dependencies.insert(point.clone());
        expected_dependencies.insert(Type::List("Vec".to_owned(), Box::new(point)));

        assert_eq!(Complex::dependencies(), expected_dependencies);
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
        fn dependencies() -> BTreeSet<Type> {
            let generics = "";
            let mut dependencies = BTreeSet::new();
            BTreeMap::<String, Vec<Point<f64>>>::add_named_type_with_dependencies_and_generics(
                &mut dependencies,
                "BTreeMap<String, Vec<Point<f64>>>",
                generics,
            );
            dependencies
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

        assert_eq!(ComplexNested::dependencies(), expected_dependencies);
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
        fn dependencies() -> BTreeSet<Type> {
            let generics = "";
            let mut dependencies = BTreeSet::new();
            Point::<Point<f64>>::add_named_type_with_dependencies_and_generics(
                &mut dependencies,
                "Point<Point<f64>>",
                generics,
            );
            dependencies
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

        assert_eq!(Recursive::dependencies(), expected_dependencies);
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
        fn dependencies() -> BTreeSet<Type> {
            let generics = "";
            let mut dependencies = BTreeSet::new();
            Vec::<Point<Point<f64>>>::add_named_type_with_dependencies_and_generics(
                &mut dependencies,
                "Vec<Point<Point<f64>>>",
                generics,
            );
            dependencies
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

        assert_eq!(NestedRecursive::dependencies(), expected_dependencies);
    }
}
