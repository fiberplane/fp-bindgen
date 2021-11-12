use super::Serializable;
use crate::types::{CustomType, Type};
use std::collections::{BTreeMap, BTreeSet};

impl Serializable for time::OffsetDateTime {
    fn name() -> String {
        "OffsetDateTime".to_owned()
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            name: "OffsetDateTime".to_owned(),
            type_args: vec![],
            rs_ty: "time::OffsetDateTime".to_owned(),
            rs_dependencies: BTreeMap::from([(
                "time".to_owned(),
                r#"{ version = "0.3", features = ["serde"] }"#.to_owned(),
            )]),
            ts_ty: "string".to_owned(),
        })
    }

    fn build_dependencies() -> BTreeSet<Type> {
        BTreeSet::new()
    }
}

impl Serializable for time::PrimitiveDateTime {
    fn name() -> String {
        "PrimitiveDateTime".to_owned()
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            name: "PrimitiveDateTime".to_owned(),
            type_args: vec![],
            rs_ty: "time::PrimitiveDateTime".to_owned(),
            rs_dependencies: BTreeMap::from([(
                "time".to_owned(),
                r#"{ version = "0.3", features = ["serde"] }"#.to_owned(),
            )]),
            ts_ty: "string".to_owned(),
        })
    }

    fn build_dependencies() -> BTreeSet<Type> {
        BTreeSet::new()
    }
}
