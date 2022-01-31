use super::Serializable;
use crate::types::{CargoDependency, CustomType, Type};
use std::collections::{BTreeMap, BTreeSet};

impl Serializable for serde_bytes::ByteBuf {
    fn name() -> String {
        "ByteBuf".to_owned()
    }

    fn build_ty() -> Type {
        Type::Custom(CustomType {
            name: "ByteBuf".to_owned(),
            type_args: vec![],
            rs_ty: "serde_bytes::ByteBuf".to_owned(),
            rs_dependencies: BTreeMap::from([(
                "serde_bytes",
                CargoDependency::with_version("0.11"),
            )]),
            serde_attrs: vec![],
            ts_ty: "ArrayBuffer".to_owned(),
            ts_declaration: None,
        })
    }

    fn build_dependencies() -> BTreeSet<Type> {
        BTreeSet::new()
    }
}
