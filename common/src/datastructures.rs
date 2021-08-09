use std::convert::TryFrom;
use syn::{Item, ItemEnum, ItemStruct};

pub enum DataStructureItem {
    Enum(ItemEnum),
    Struct(ItemStruct),
}

impl DataStructureItem {
    pub fn name(&self) -> String {
        match self {
            Self::Enum(item) => item.ident.to_string(),
            Self::Struct(item) => item.ident.to_string(),
        }
    }
}

impl TryFrom<Item> for DataStructureItem {
    type Error = String;

    fn try_from(value: Item) -> Result<Self, Self::Error> {
        match value {
            Item::Enum(item) => Ok(Self::Enum(item)),
            Item::Struct(item) => Ok(Self::Struct(item)),
            item => Err(format!(
                "Only struct and enum are supported as data structure. Found: {:?}",
                item
            )),
        }
    }
}
