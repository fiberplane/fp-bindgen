use crate::types::{resolve_type, GenericArgument, Type};
use std::collections::BTreeSet;
use syn::{AngleBracketedGenericArguments, Path, PathArguments};

/// Specializes a type with optional generic arguments, including its dependencies.
///
/// `name` is the name of the type as seen by the type that instantiates it. The instantiating
/// type may contain additional type information that is unknown to the generic type itself,
/// which gives us the opportunity to specialize.
///
/// `dependencies` are dependencies of the type being specialized. Whenever we need to look up
/// a concrete type for a generic argument, it is assumed to be part of these dependencies.
///
/// # Example
///
/// ```
/// use fp_bindgen::generics::*;
/// use fp_bindgen::primitives::*;
/// use fp_bindgen::types::*;
/// use std::collections::BTreeSet;
///
/// let ty = Type::Struct(
///     "Point".to_owned(),
///     vec![GenericArgument { name: "T".to_owned(), ty: None }],
///     vec![],
///     vec![Field {
///         name: "value".to_owned(),
///         ty: Type::GenericArgument(Box::new(GenericArgument {
///             name: "T".to_owned(),
///             ty: None
///         })),
///         doc_lines: vec![],
///         attrs: FieldAttrs::default(),
///     }],
///     StructOptions::default(),
/// );
///
/// let mut dependencies = BTreeSet::new();
/// dependencies.insert(Type::Primitive(Primitive::F64));
///
/// let mut expected_types = BTreeSet::new();
/// expected_types.insert(ty.clone().with_specialized_args(&[GenericArgument {
///     name: "T".to_owned(),
///     ty: Some(Type::Primitive(Primitive::F64)),
/// }]));
/// expected_types.insert(Type::Primitive(Primitive::F64));
///
/// assert_eq!(
///     specialize_type_with_dependencies(ty, "Point<f64>", &dependencies),
///     expected_types
/// );
/// ```
pub fn specialize_type_with_dependencies(
    ty: Type,
    name: &str,
    dependencies: &BTreeSet<Type>,
) -> BTreeSet<Type> {
    let mut specialized_types = BTreeSet::new();

    if contains_generic_arg(name) {
        let args = extract_args(name);
        for dependency in dependencies {
            let dependency_name = dependency.name();
            for arg in args.iter() {
                if peel(arg).0 == peel(&dependency_name).0 {
                    specialized_types.append(&mut specialize_type_with_dependencies(
                        dependency.clone(),
                        arg,
                        dependencies,
                    ));
                }
            }
        }
    } else {
        for dependency in dependencies {
            specialized_types.insert(dependency.clone());
        }
    }

    let ty_name = ty.name();
    let generic_args = peel(&ty_name).1;
    let specialized_args = parse_generic_args_for_type(generic_args, &ty, name, &specialized_types);
    specialized_types.insert(ty.with_specialized_args(&specialized_args));

    specialized_types
}

pub fn contains_generic_arg(name: &str) -> bool {
    name.contains('<')
}

pub fn extract_args(name: &str) -> Vec<String> {
    let generic_args = syn::parse_str::<AngleBracketedGenericArguments>(peel(name).1).unwrap();
    generic_args
        .args
        .iter()
        .filter_map(|param| match param {
            syn::GenericArgument::Type(syn::Type::Path(generic_ty)) => Some(generic_ty),
            _ => None,
        })
        .map(|generic_arg| get_name_from_path(&generic_arg.path))
        .collect()
}

pub fn extract_args_str(name: &str) -> &str {
    let args_start_index = name.find('<').expect("Generic brackets expected");
    let args_end_index = name.rfind('>').expect("Generic brackets expected");
    &name[args_start_index + 1..args_end_index]
}

fn get_name_from_path(path: &Path) -> String {
    path.segments
        .last()
        .map(|segment| match &segment.arguments {
            PathArguments::None => segment.ident.to_string(),
            PathArguments::AngleBracketed(bracketed) => format!(
                "{}<{}>",
                segment.ident,
                bracketed
                    .args
                    .iter()
                    .map(|arg| match arg {
                        syn::GenericArgument::Type(syn::Type::Path(path))
                            if path.qself.is_none() =>
                            get_name_from_path(&path.path),
                        _ => panic!("Unsupported generic argument in path: {:?}", path),
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            PathArguments::Parenthesized(_) => {
                panic!("Unsupported arguments in path: {:?}", path)
            }
        })
        .unwrap_or_else(String::new)
}

fn parse_generic_args_for_type(
    generic_args: &str,
    ty: &Type,
    name: &str,
    dependencies: &BTreeSet<Type>,
) -> Vec<GenericArgument> {
    match ty {
        // Some of our types contain "implicit" generics, mainly container types
        // we treat specially. For those, we just pass the buck to their inner
        // types:
        Type::Container(_, item) | Type::List(_, item) | Type::Map(_, _, item) => {
            return parse_generic_args_for_type(
                peel(&item.name()).1,
                item.as_ref(),
                if contains_generic_arg(name) {
                    extract_args_str(name)
                } else {
                    ""
                },
                dependencies,
            );
        }
        _ => {
            if generic_args.is_empty() {
                return Vec::new();
            }
        }
    }

    let resolve_name = |name| {
        let (name, args) = if contains_generic_arg(name) {
            peel(name)
        } else {
            (name, "")
        };
        let ty = syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(syn::PathSegment {
                ident: syn::Ident::new(name, proc_macro2::Span::call_site()),
                arguments: if args.is_empty() {
                    syn::PathArguments::None
                } else {
                    syn::PathArguments::AngleBracketed(syn::parse_str(args).unwrap())
                },
            }),
        });
        resolve_type(&ty, dependencies)
    };

    let generic_args = syn::parse_str::<AngleBracketedGenericArguments>(generic_args).unwrap();
    let generic_args = generic_args.args.iter().filter_map(|param| match param {
        syn::GenericArgument::Type(syn::Type::Path(generic_ty)) => Some(generic_ty),
        _ => None,
    });
    match ty {
        Type::Enum(_, args, _, _, _) | Type::Struct(_, args, _, _, _) => {
            let generic_args_from_name = peel(name).1;
            if generic_args_from_name.is_empty() {
                return args.clone();
            }

            let names = generic_args_from_name[1..generic_args_from_name.len() - 1]
                .split(',')
                .map(str::trim);
            args.iter()
                .zip(generic_args.zip(names))
                .map(|(arg, (generic_arg, name))| GenericArgument {
                    name: arg.name.clone(),
                    ty: if generic_arg
                        .path
                        .get_ident()
                        .map(|ident| *ident == arg.name)
                        .unwrap_or_default()
                    {
                        resolve_name(name)
                    } else {
                        None
                    },
                })
                .collect()
        }
        Type::GenericArgument(arg) => generic_args
            .map(|generic_arg| GenericArgument {
                name: arg.name.clone(),
                ty: if generic_arg
                    .path
                    .get_ident()
                    .map(|ident| *ident == arg.name)
                    .unwrap_or_default()
                {
                    match arg.ty.as_ref() {
                        Some(ty) => Some(ty.clone()),
                        None => resolve_name(name),
                    }
                } else {
                    None
                },
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Peels the name off generic arguments.
///
/// ## Examples
///
/// ```
/// use fp_bindgen::generics::*;
///
/// assert_eq!(peel("Point<T>"), ("Point", "<T>"));
/// assert_eq!(peel("ConcreteType"), ("ConcreteType", ""));
/// ```
pub fn peel(name: &str) -> (&str, &str) {
    match (name.find('<'), name.rfind('>')) {
        (Some(start_index), Some(end_index)) => (
            name[0..start_index].trim(),
            &name[start_index..end_index + 1],
        ),
        _ => (name, ""),
    }
}
