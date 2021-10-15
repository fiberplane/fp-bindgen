use syn::Generics;

use crate::types::{resolve_type, GenericArgument, Type};
use std::collections::BTreeSet;

/// Specializes a type with optional generic arguments, including its dependencies.
///
/// `name` is the name of the type as seen by the type that instantiates it. The instantiating
/// type may contain additional type information that is unknown to the generic type itself,
/// which gives us the opportunity to specialize.
///
/// `generic_args` is the definition of the generic arguments that require specialization.
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
///     specialize_type_with_dependencies(ty, "Point<f64>", "<T>", &dependencies),
///     expected_types
/// );
/// ```
pub fn specialize_type_with_dependencies(
    ty: Type,
    name: &str,
    generic_args: &str,
    dependencies: &BTreeSet<Type>,
) -> BTreeSet<Type> {
    let mut specialized_types = BTreeSet::new();

    for dependency in dependencies {
        if contains_generic_arg(name) {
            let dependency_name = dependency.name();
            for arg in extract_args(name).split(',').map(str::trim) {
                let (name, _) = peel(arg);
                let (dependency_name, dependency_args) = peel(&dependency_name);
                if name == dependency_name {
                    specialized_types.append(&mut specialize_type_with_dependencies(
                        dependency.clone(),
                        arg,
                        dependency_args,
                        dependencies,
                    ));
                }
            }
        } else {
            specialized_types.insert(dependency.clone());
        }
    }

    let generic_args = parse_generic_args_for_type(generic_args, &ty, name, &specialized_types);
    specialized_types.insert(ty.with_specialized_args(&generic_args));

    specialized_types
}

pub fn contains_generic_arg(name: &str) -> bool {
    name.contains('<')
}

pub fn extract_args(name: &str) -> &str {
    let args_start_index = name.find('<').expect("Generic brackets expected");
    let args_end_index = name.rfind('>').expect("Generic brackets expected");
    &name[args_start_index + 1..args_end_index]
}

fn parse_generic_args_for_type(
    generic_args: &str,
    ty: &Type,
    name: &str,
    dependencies: &BTreeSet<Type>,
) -> Vec<GenericArgument> {
    if generic_args.is_empty() {
        // Some of our types contain "implicit" generics, mainly container types
        // we treat specially. For those, we just pass the buck to their inner
        // types:
        return match ty {
            Type::Container(_, item) | Type::List(_, item) | Type::Map(_, _, item) => {
                parse_generic_args_for_type(
                    peel(&item.name()).1,
                    item.as_ref(),
                    if contains_generic_arg(name) {
                        extract_args(name)
                    } else {
                        ""
                    },
                    dependencies,
                )
            }
            _ => Vec::new(),
        };
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

    let generic_args = syn::parse_str::<Generics>(generic_args).unwrap();
    let generic_params = generic_args.params.iter().filter_map(|param| match param {
        syn::GenericParam::Type(generic_ty) => Some(generic_ty),
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
                .zip(generic_params.zip(names))
                .map(|(arg, (param, name))| GenericArgument {
                    name: arg.name.clone(),
                    ty: if param.ident == arg.name {
                        resolve_name(name)
                    } else {
                        None
                    },
                })
                .collect()
        }
        Type::GenericArgument(arg) => generic_params
            .map(|param| GenericArgument {
                name: arg.name.clone(),
                ty: if param.ident == arg.name {
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
/// ```rs
/// assert_eq!(peel("Point<T>"), ("Point", "<T>"));
/// assert_eq!(peel("ConcreteType"), ("ConcreteType", ""));
/// ```
fn peel(name: &str) -> (&str, &str) {
    match (name.find('<'), name.rfind('>')) {
        (Some(start_index), Some(end_index)) => (
            name[0..start_index].trim(),
            &name[start_index..end_index + 1],
        ),
        _ => (name, ""),
    }
}
