use std::{collections::BTreeMap, convert::TryFrom, str::FromStr};
use syn::{FnArg, ForeignItem, ForeignItemFn, Path, Type, __private::ToTokens};

/// Maps from function name to the stringified function declaration.
#[derive(Debug, Default)]
pub struct FunctionMap(BTreeMap<String, String>);

impl FunctionMap {
    pub fn insert(&mut self, key: String, value: String) {
        self.0.insert(key, value);
    }

    pub fn insert_str(&mut self, key: &str, value: &str) {
        self.0.insert(key.to_owned(), value.to_owned());
    }

    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
}

impl IntoIterator for FunctionMap {
    type Item = (String, String);
    type IntoIter = std::collections::btree_map::IntoIter<String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a FunctionMap {
    type Item = (&'a String, &'a String);
    type IntoIter = std::collections::btree_map::Iter<'a, String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

pub struct Function(ForeignItemFn);

impl Function {
    pub fn name(&self) -> String {
        self.0.sig.ident.to_string()
    }

    pub fn args(&self) -> Vec<FunctionArg> {
        self.0
            .sig
            .inputs
            .iter()
            .map(|input| match input {
                FnArg::Receiver(_) => panic!(
                    "Methods are not supported. Found `self` in function declaration: {:?}",
                    self.0
                ),
                FnArg::Typed(arg) => FunctionArg {
                    name: arg.pat.to_token_stream().to_string(),
                    type_path: match arg.ty.as_ref() {
                        Type::Path(path) if path.qself.is_none() => path.path.clone(),
                        _ => panic!(
                            "Only plain value types are supported. \
                                    Incompatible type in function declaration: {:?}",
                            self.0
                        ),
                    },
                },
            })
            .collect()
    }

    pub fn is_async(&self) -> bool {
        self.0.sig.asyncness.is_some()
    }
}

impl FromStr for Function {
    type Err = String;

    fn from_str(function_decl: &str) -> Result<Self, Self::Err> {
        Self::try_from(syn::parse_str::<ForeignItem>(function_decl).unwrap())
    }
}

impl TryFrom<ForeignItem> for Function {
    type Error = String;

    fn try_from(item: ForeignItem) -> Result<Self, Self::Error> {
        match item {
            ForeignItem::Fn(item) => Ok(Self(item)),
            item => Err(format!(
                "Only functions can be imported or exported. Found: {:?}",
                item
            )),
        }
    }
}

pub struct FunctionArg {
    pub name: String,
    pub type_path: Path,
}
