use crate::primitives::Primitive;
use std::{convert::TryFrom, str::FromStr};
use syn::{Item, ItemEnum, ItemStruct};

pub enum Type {
    Enum(ItemEnum),
    Primitive(Primitive),
    Struct(ItemStruct),
}

impl Type {
    pub fn name(&self) -> String {
        match self {
            Self::Enum(item) => item.ident.to_string(),
            Self::Primitive(primitive) => primitive.name(),
            Self::Struct(item) => item.ident.to_string(),
        }
    }
}

impl FromStr for Type {
    type Err = String;

    fn from_str(type_decl: &str) -> Result<Self, Self::Err> {
        Self::try_from(syn::parse_str::<Item>(type_decl).unwrap())
    }
}

impl TryFrom<Item> for Type {
    type Error = String;

    fn try_from(item: Item) -> Result<Self, Self::Error> {
        match item {
            Item::Enum(item) => Ok(Self::Enum(item)),
            Item::Struct(item) => Ok(Self::Struct(item)),
            item => Err(format!(
                "Only struct and enum types can be constructed from an item. Found: {:?}",
                item
            )),
        }
    }
}
