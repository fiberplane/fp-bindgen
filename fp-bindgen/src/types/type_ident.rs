use super::is_runtime_bound;
use crate::primitives::Primitive;
use std::num::NonZeroUsize;
use std::{convert::TryFrom, fmt::Display, str::FromStr};
use syn::{PathArguments, TypeParamBound, TypePath, TypeTuple};

#[derive(Clone, Default, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct TypeIdent {
    pub name: String,
    pub generic_args: Vec<(TypeIdent, Vec<String>)>,
    /// If this TypeIdent represents an array this field will store the length
    pub array: Option<NonZeroUsize>,
}

impl TypeIdent {
    pub fn new(name: impl Into<String>, generic_args: Vec<(TypeIdent, Vec<String>)>) -> Self {
        Self {
            name: name.into(),
            generic_args,
            array: None,
        }
    }

    pub fn is_array(&self) -> bool {
        self.array.is_some()
    }

    pub fn is_primitive(&self) -> bool {
        self.as_primitive().is_some()
    }

    pub fn as_primitive(&self) -> Option<Primitive> {
        if self.array.is_none() {
            Primitive::from_str(&self.name).ok()
        } else {
            None
        }
    }

    pub fn format(&self, include_bounds: bool) -> String {
        let ty = if self.generic_args.is_empty() {
            self.name.clone()
        } else {
            format_args!(
                "{}<{}>",
                self.name,
                self.generic_args
                    .iter()
                    .map(|(arg, bounds)| {
                        if bounds.is_empty() || !include_bounds {
                            format!("{arg}")
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
        };

        match self.array {
            Some(len) => format!("[{ty}; {len}]"),
            None => ty,
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
        Self::from_str(name)
            .unwrap_or_else(|_| panic!("Could not convert '{}' into a TypeIdent", name))
    }
}

impl FromStr for TypeIdent {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let (string, array) = if string.starts_with('[') {
            // Remove brackets and split on ;
            let split = string
                .strip_prefix('[')
                .and_then(|s| s.strip_suffix(']'))
                .ok_or(format!("Invalid array syntax in: {string}"))?
                .split(';')
                .collect::<Vec<_>>();

            let element = split[0].trim();
            let len = usize::from_str(split[1].trim())
                .map_err(|_| format!("Invalid array length in: {string}"))?;

            let primitive = Primitive::from_str(element)?;
            if primitive.js_array_name().is_none() {
                return Err(format!(
                    "Only arrays of primitives supported by Javascript are allowed, found: {string}"
                ));
            }

            (element, NonZeroUsize::new(len))
        } else {
            (string, None)
        };

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
                array,
            })
        } else {
            Ok(Self {
                name: string.into(),
                generic_args: vec![],
                array,
            })
        }
    }
}

impl From<String> for TypeIdent {
    fn from(name: String) -> Self {
        Self {
            name,
            generic_args: Vec::new(),
            ..Default::default()
        }
    }
}

impl Ord for TypeIdent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // We only compare the name and array so that any type is only included once in
        // a map, regardless of how many concrete instances are used with
        // different generic arguments.
        (&self.name, self.array).cmp(&(&other.name, other.array))
    }
}

impl PartialOrd for TypeIdent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // We only compare the name and array so that any type is only included once in
        // a map, regardless of how many concrete instances are used with
        // different generic arguments.
        (&self.name, self.array).partial_cmp(&(&other.name, other.array))
    }
}

impl TryFrom<&syn::Type> for TypeIdent {
    type Error = String;

    fn try_from(ty: &syn::Type) -> Result<Self, Self::Error> {
        match ty {
            syn::Type::Array(syn::TypeArray {
                elem,
                len: syn::Expr::Lit(syn::ExprLit { lit, .. }),
                ..
            }) => {
                let array_len = match lit {
                    syn::Lit::Int(int) => int.base10_digits().parse::<usize>(),
                    _ => panic!(),
                }
                .unwrap();
                let elem_ident = TypeIdent::try_from(elem.as_ref())?;

                Ok(Self {
                    name: elem_ident.name,
                    generic_args: vec![],
                    array: NonZeroUsize::new(array_len),
                })
            }
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
                                                "Lifecycle bounds are not supported: {bound:?}"
                                            )),
                                        })
                                        .collect::<Vec<_>>();
                                    for bound in bounds {
                                        generic_arg_bounds.push(bound?);
                                    }
                                }
                                arg => {
                                    return Err(format!("Unsupported generic argument: {arg:?}"));
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
                    ..Default::default()
                })
            }
            syn::Type::Tuple(TypeTuple {
                elems,
                paren_token: _,
            }) if elems.is_empty() => Ok(TypeIdent::from("()")),
            ty => Err(format!("Unsupported type: {ty:?}")),
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

    #[test]
    fn type_ident_from_str_array() {
        let t = TypeIdent::from_str("[u32; 8]").unwrap();
        assert_eq!(t.name, "u32");
        assert!(t.generic_args.is_empty());
        assert_eq!(t.array, NonZeroUsize::new(8));

        // Cannot create non-primitive arrays, and other error scenarios
        assert!(TypeIdent::from_str("[Vec<f32>; 8]").is_err());
        assert!(TypeIdent::from_str("[u32;]").is_err());
        assert!(TypeIdent::from_str("[u32; foo]").is_err());
        assert!(TypeIdent::from_str("[u32; -1]").is_err());

        // Unsupported primitive array types
        assert!(TypeIdent::from_str("[u64; 8]").is_err());
    }
}
