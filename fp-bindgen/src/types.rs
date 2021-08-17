use crate::primitives::Primitive;
use std::{convert::TryFrom, str::FromStr};
use syn::{Item, ItemEnum, ItemStruct};

#[derive(Clone, Debug, Eq, PartialEq)]
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
        match Primitive::from_str(type_decl) {
            Ok(primitive) => Ok(Self::Primitive(primitive)),
            Err(_) => Self::try_from(syn::parse_str::<Item>(type_decl).unwrap()),
        }
    }
}

impl Ord for Type {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name().cmp(&other.name())
    }
}

impl PartialOrd for Type {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name().partial_cmp(&other.name())
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
