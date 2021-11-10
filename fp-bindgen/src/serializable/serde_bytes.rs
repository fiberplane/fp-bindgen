use super::Serializable;
use crate::types::{CustomType, Type};
use std::collections::{BTreeMap, BTreeSet};

impl Serializable for serde_bytes::ByteBuf {
    fn name() -> String {
        "serde_bytes::ByteBuf".to_owned()
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            name: "ByteBuf".to_owned(),
            type_args: vec![],
            rs_ty: "serde_bytes::ByteBuf".to_owned(),
            rs_dependencies: BTreeMap::from([("serde_bytes".to_owned(), r#""0.11""#.to_owned())]),
            ts_ty: "ArrayBuffer".to_owned(),
        })
    }

    fn build_dependencies() -> BTreeSet<Type> {
        BTreeSet::new()
    }
}
