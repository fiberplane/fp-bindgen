use once_cell::unsync::Lazy;
use std::{collections::BTreeMap, convert::TryFrom, sync::Mutex};
use syn::{Item, ItemFn};

pub static FUNCTION_IMPORTS: Lazy<Mutex<BTreeMap<String, FunctionItem>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

pub static FUNCTION_EXPORTS: Lazy<Mutex<BTreeMap<String, FunctionItem>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

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
