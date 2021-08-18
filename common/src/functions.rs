use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use std::{collections::BTreeMap, convert::TryFrom, str::FromStr};
use syn::{FnArg, ForeignItem, ForeignItemFn, Path, Type};

/// Maps from function name to the stringified function declaration.
#[derive(Debug, Default)]
pub struct FunctionMap(BTreeMap<String, String>);

impl FunctionMap {
    pub fn from_stream(stream: TokenStream) -> Self {
        let mut functions = Self::new();
        let mut current_item_tokens = Vec::<TokenTree>::new();
        for token in stream.into_iter() {
            match token {
                TokenTree::Punct(punct) if punct.as_char() == ';' => {
                    current_item_tokens.push(TokenTree::Punct(punct));

                    let stream = current_item_tokens.into_iter().collect::<TokenStream>();
                    let function =
                        FunctionItem::try_from(syn::parse2::<ForeignItem>(stream).unwrap())
                            .unwrap();
                    functions.insert(function.name(), function.to_token_stream().to_string());
                    current_item_tokens = Vec::new();
                }
                other => current_item_tokens.push(other),
            }
        }
        functions
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.0.insert(key, value);
    }

    pub fn insert_str(&mut self, key: &str, value: &str) {
        self.0.insert(key.to_owned(), value.to_owned());
    }

    pub fn keys(&self) -> std::collections::btree_map::Keys<String, String> {
        self.0.keys()
    }

    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn values(&self) -> std::collections::btree_map::Values<String, String> {
        self.0.values()
    }
}

impl IntoIterator for FunctionMap {
    type Item = (String, String);
    type IntoIter = std::collections::btree_map::IntoIter<String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub enum FunctionItem {
    Function(ForeignItemFn),
}

impl FunctionItem {
    pub fn name(&self) -> String {
        match self {
            Self::Function(item) => item.sig.ident.to_string(),
        }
    }

    pub fn args(&self) -> Vec<FunctionArg> {
        match self {
            Self::Function(item) => item
                .sig
                .inputs
                .iter()
                .map(|input| match input {
                    FnArg::Receiver(_) => panic!(
                        "Methods are not supported. Found `self` in function declaration: {:?}",
                        item
                    ),
                    FnArg::Typed(arg) => FunctionArg {
                        name: arg.pat.to_token_stream().to_string(),
                        type_path: match arg.ty.as_ref() {
                            Type::Path(path) if path.qself.is_none() => path.path.clone(),
                            _ => panic!(
                                "Only plain value types are supported. \
                                    Incompatible type in function declaration: {:?}",
                                item
                            ),
                        },
                    },
                })
                .collect(),
        }
    }

    pub fn is_async(&self) -> bool {
        match self {
            Self::Function(item) => item.sig.asyncness.is_some(),
        }
    }
}

impl FromStr for FunctionItem {
    type Err = String;

    fn from_str(function_decl: &str) -> Result<Self, Self::Err> {
        Self::try_from(syn::parse_str::<ForeignItem>(function_decl).unwrap())
    }
}

impl ToTokens for FunctionItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Function(item) => item.to_tokens(tokens),
        }
    }
}

impl TryFrom<ForeignItem> for FunctionItem {
    type Error = String;

    fn try_from(value: ForeignItem) -> Result<Self, Self::Error> {
        match value {
            ForeignItem::Fn(item) => Ok(Self::Function(item)),
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
