use super::Serializable;
use crate::types::{CargoDependency, CustomType, Type};
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
                "time",
                CargoDependency {
                    branch: None,
                    git: None,
                    path: None,
                    version: Some("0.3"),
                    features: BTreeSet::from(["serde-human-readable"]),
                },
            )]),
            ts_ty: "string".to_owned(),
            ts_declaration: None,
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
                "time",
                CargoDependency {
                    branch: None,
                    git: None,
                    path: None,
                    version: Some("0.3"),
                    features: BTreeSet::from(["serde-human-readable"]),
                },
            )]),
            ts_ty: "string".to_owned(),
            ts_declaration: None,
        })
    }

    fn build_dependencies() -> BTreeSet<Type> {
        BTreeSet::new()
    }
}
