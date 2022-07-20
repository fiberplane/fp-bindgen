use crate::primitives::Primitive;
use std::{
    convert::{Infallible, TryFrom},
    fmt::Display,
    str::FromStr,
};
use syn::{PathArguments, TypeParamBound, TypePath, TypeTuple};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct TypeIdent {
    pub name: String,
    pub generic_args: Vec<(TypeIdent, Vec<String>)>,
}

impl TypeIdent {
    pub fn new(name: impl Into<String>, generic_args: Vec<(TypeIdent, Vec<String>)>) -> Self {
        Self {
            name: name.into(),
            generic_args,
        }
    }

    pub fn is_primitive(&self) -> bool {
        Primitive::from_str(&self.name).is_ok()
    }

    pub fn format(&self, include_bounds: bool) -> String {
        if self.generic_args.is_empty() {
            self.name.clone()
        } else {
            format_args!(
                "{}<{}>",
                self.name,
                self.generic_args
                    .iter()
                    .map(|(arg, bounds)| {
                        if bounds.is_empty() || !include_bounds {
                            format!("{}", arg)
                        } else {
                            format!(
                                "{}: {}",
                                arg,
                                bounds
                                    .iter()
                                    .filter(|b| is_runtime_bound(b))
                                    .cloned()
                                    .collect::<Vec<_>>()
                                    .join(" + ")
                            )
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .to_string()
        }
    }
}

impl Display for TypeIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.format(true))
    }
}

impl From<&str> for TypeIdent {
    fn from(name: &str) -> Self {
        Self::from(name.to_owned())
    }
}

impl FromStr for TypeIdent {
    type Err = Infallible;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(start_index) = string.find('<') {
            let end_index = string.rfind('>').unwrap_or(string.len());
            Ok(Self {
                name: string[0..start_index]
                    .trim_end_matches(|c: char| c.is_whitespace() || c == ':')
                    .to_owned(),
                generic_args: string[start_index + 1..end_index]
                    .split(',')
                    .into_iter()
                    .map(|arg| {
                        let (arg, bounds) = arg.split_once(':').unwrap_or((arg, ""));
                        let ident = Self::from_str(arg.trim());
                        let bounds = bounds
                            .split('+')
                            .map(|b| b.trim().to_string())
                            .filter(|b| !b.is_empty())
                            .collect();
                        ident.map(|ident| (ident, bounds))
                    })
                    .collect::<Result<Vec<(Self, Vec<String>)>, Self::Err>>()?,
            })
        } else {
            Ok(Self::from(string))
        }
    }
}

impl From<String> for TypeIdent {
    fn from(name: String) -> Self {
        Self {
            name,
            generic_args: Vec::new(),
        }
    }
}

impl Ord for TypeIdent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // We only compare the name so that any type is only included once in
        // a map, regardless of how many concrete instances are used with
        // different generic arguments.
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for TypeIdent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // We only compare the name so that any type is only included once in
        // a map, regardless of how many concrete instances are used with
        // different generic arguments.
        self.name.partial_cmp(&other.name)
    }
}

impl TryFrom<&syn::Type> for TypeIdent {
    type Error = String;

    fn try_from(ty: &syn::Type) -> Result<Self, Self::Error> {
        match ty {
            syn::Type::Path(TypePath { path, qself }) if qself.is_none() => {
                let mut generic_args = vec![];
                if let Some(segment) = path.segments.last() {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        for arg in &args.args {
                            let generic_arg_ident;
                            let mut generic_arg_bounds = vec![];
                            match arg {
                                syn::GenericArgument::Type(ty) => {
                                    generic_arg_ident = Some(TypeIdent::try_from(ty)?);
                                }
                                syn::GenericArgument::Constraint(cons) => {
                                    generic_arg_ident =
                                        Some(TypeIdent::new(cons.ident.to_string(), vec![]));

                                    let bounds = cons
                                        .bounds
                                        .iter()
                                        .map(|bound| match bound {
                                            TypeParamBound::Trait(tr) => {
                                                Ok(path_to_string(&tr.path))
                                            }
                                            TypeParamBound::Lifetime(_) => Err(format!(
                                                "Lifecycle bounds are not supported: {:?}",
                                                bound
                                            )),
                                        })
                                        .collect::<Vec<_>>();
                                    for bound in bounds {
                                        generic_arg_bounds.push(bound?);
                                    }
                                }
                                arg => {
                                    return Err(format!("Unsupported generic argument: {:?}", arg));
                                }
                            }
                            if let Some(ident) = generic_arg_ident {
                                generic_args.push((ident, generic_arg_bounds));
                            }
                        }
                    }
                }

                Ok(Self {
                    name: path_to_string(path),
                    generic_args,
                })
            }
            syn::Type::Tuple(TypeTuple {
                elems,
                paren_token: _,
            }) if elems.is_empty() => Ok(TypeIdent::from("()")),
            ty => Err(format!("Unsupported type: {:?}", ty)),
        }
    }
}

fn path_to_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

// Used to remove the 'Serializable' bound from generated types, since this trait only exists in fp-bindgen
// and doesn't exist at runtime.
fn is_runtime_bound(bound: &str) -> bool {
    // Filtering by string is a bit dangerous since users may have their own 'Serializable' trait :(
    bound != "Serializable" && bound != "fp_bindgen::prelude::Serializable"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_ident_from_syn_type() {
        let ty = syn::parse_str::<syn::Type>("u32").unwrap();
        let t = TypeIdent::try_from(&ty).unwrap();
        assert_eq!(t.name, "u32");
        assert!(t.generic_args.is_empty());

        let ty = syn::parse_str::<syn::Type>("Vec<u32>").unwrap();
        let t = TypeIdent::try_from(&ty).unwrap();
        assert_eq!(t.name, "Vec");
        assert_eq!(
            t.generic_args,
            vec![(TypeIdent::new("u32", vec![]), vec![])]
        );

        let ty = syn::parse_str::<syn::Type>("BTreeMap<K, V>").unwrap();
        let t = TypeIdent::try_from(&ty).unwrap();
        assert_eq!(t.name, "BTreeMap");
        assert_eq!(
            t.generic_args,
            vec![
                (TypeIdent::new("K", vec![]), vec![]),
                (TypeIdent::new("V", vec![]), vec![])
            ]
        );

        let ty = syn::parse_str::<syn::Type>("Vec<T: Debug + Display>").unwrap();
        let t = TypeIdent::try_from(&ty).unwrap();
        assert_eq!(t.name, "Vec");
        assert_eq!(
            t.generic_args,
            vec![(
                TypeIdent::new("T", vec![]),
                vec!["Debug".into(), "Display".into()]
            )]
        );
    }

    #[test]
    fn type_ident_from_str() {
        let t = TypeIdent::from_str("u32").unwrap();
        assert_eq!(t.name, "u32");
        assert!(t.generic_args.is_empty());

        let t = TypeIdent::from_str("Vec<u32>").unwrap();
        assert_eq!(t.name, "Vec");
        assert_eq!(
            t.generic_args,
            vec![(TypeIdent::new("u32", vec![]), vec![])]
        );

        let t = TypeIdent::from_str("BTreeMap<K, V>").unwrap();
        assert_eq!(t.name, "BTreeMap");
        assert_eq!(
            t.generic_args,
            vec![
                (TypeIdent::new("K", vec![]), vec![]),
                (TypeIdent::new("V", vec![]), vec![])
            ]
        );

        let t = TypeIdent::from_str("Vec<T: Debug + Display>").unwrap();
        assert_eq!(t.name, "Vec");
        assert_eq!(
            t.generic_args,
            vec![(
                TypeIdent::new("T", vec![]),
                vec!["Debug".into(), "Display".into()]
            )]
        );
    }
}
