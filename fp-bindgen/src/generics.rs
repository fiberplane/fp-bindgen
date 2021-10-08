use crate::types::{GenericArgument, Type};
use std::collections::BTreeSet;

/// Specializes a type with optional generic arguments, including its dependencies.
///
/// `name` is the name of the type as seen by the type that instantiates it. The instantiating
/// type may contain additional type information that is unknown to the generic type itself,
/// which gives us the opportunity to specialize.
///
/// `generic_args` is the definition of the generic arguments of the instantiating type,
pub fn specialize_type_with_dependencies(
    ty: Type,
    name: &str,
    generic_args: &str,
    dependencies: &BTreeSet<Type>,
) -> BTreeSet<Type> {
    println!(
        "Specializing type: {}, with name: {} and generic args: {}",
        ty.name(),
        name,
        generic_args
    );
    let generic_args = parse_generic_args_for_type(generic_args, &ty, name);
    println!("Parsed args: {:?}", generic_args);

    let mut specialized_types = BTreeSet::new();
    specialized_types.insert(ty.with_specialized_args(&generic_args));

    for dependency in dependencies {
        println!("Let me also specialize dependency: {}", dependency.name());
        if contains_generic_arg(name) {
            let dependency_name = dependency.name();
            for arg in extract_args(name) {
                let (name, _) = peel(arg);
                let (dependency_name, dependency_args) = peel(&dependency_name);
                println!("{} == {}", name, dependency_name);
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
            println!("Specialize those args: {:?}", generic_args);
            specialized_types.insert(dependency.clone().with_specialized_args(&generic_args));
        }
    }

    specialized_types
}

pub fn contains_generic_arg(name: &str) -> bool {
    name.contains('<')
}

pub fn extract_args(name: &str) -> Vec<&str> {
    let args_start_index = name.find('<').expect("Generic brackets expected");
    let args_end_index = name.rfind('>').expect("Generic brackets expected");
    name[args_start_index + 1..args_end_index]
        .split(',')
        .map(|arg| arg.trim())
        .collect()
}

fn parse_generic_args_for_type(generic_args: &str, ty: &Type, name: &str) -> Vec<GenericArgument> {
    let generic_args = if generic_args.is_empty() {
        syn::Generics::default()
    } else {
        syn::parse_str(generic_args).unwrap()
    };
    generic_args
        .params
        .iter()
        .filter_map(|param| match param {
            syn::GenericParam::Type(generic_ty) => Some(GenericArgument {
                name: generic_ty.ident.to_string(),
                ty: if generic_ty.ident == name {
                    Some(ty.clone())
                } else {
                    None
                },
            }),
            _ => None,
        })
        .collect::<Vec<_>>()
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
