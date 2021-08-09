use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::ToTokens;
use std::{collections::BTreeMap, convert::TryFrom, str::FromStr};
use syn::{Item, ItemFn};

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
                    // To be parseable as an `ItemFn`, the function should have an empty body:
                    current_item_tokens.push(TokenTree::Group(Group::new(
                        Delimiter::Brace,
                        TokenStream::new(),
                    )));

                    let stream = current_item_tokens.into_iter().collect::<TokenStream>();
                    let function =
                        FunctionItem::try_from(syn::parse2::<Item>(stream).unwrap()).unwrap();
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

pub enum FunctionItem {
    Function(ItemFn),
}

impl FunctionItem {
    pub fn name(&self) -> String {
        match self {
            Self::Function(item) => item.sig.ident.to_string(),
        }
    }
}

impl FromStr for FunctionItem {
    type Err = String;

    fn from_str(function_decl: &str) -> Result<Self, Self::Err> {
        Self::try_from(syn::parse_str::<Item>(function_decl).unwrap())
    }
}

impl ToTokens for FunctionItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Function(item) => item.to_tokens(tokens),
        }
    }
}

impl TryFrom<Item> for FunctionItem {
    type Error = String;

    fn try_from(value: Item) -> Result<Self, Self::Error> {
        match value {
            Item::Fn(item) => Ok(Self::Function(item)),
            item => Err(format!(
                "Only functions can be imported or exported. Found: {:?}",
                item
            )),
        }
    }
}
