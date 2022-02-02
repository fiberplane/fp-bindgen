use super::Serializable;
use crate::types::{CargoDependency, CustomType, Type, TypeIdent};
use std::collections::{BTreeMap, BTreeSet};

impl Serializable for time::OffsetDateTime {
    fn ident() -> TypeIdent {
        TypeIdent::from("OffsetDateTime".to_owned())
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            ident: Self::ident(),
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
            serde_attrs: vec![],
            ts_ty: "string".to_owned(),
            ts_declaration: None,
        })
    }
}

impl Serializable for time::PrimitiveDateTime {
    fn ident() -> TypeIdent {
        TypeIdent::from("PrimitiveDateTime".to_owned())
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            ident: Self::ident(),
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
            serde_attrs: vec![],
            ts_ty: "string".to_owned(),
            ts_declaration: None,
        })
    }
}
