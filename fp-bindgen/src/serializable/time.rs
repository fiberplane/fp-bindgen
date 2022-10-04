use super::Serializable;
use crate::types::{CargoDependency, CustomType, Type, TypeIdent};
use std::collections::{BTreeMap, BTreeSet};

impl Serializable for time::OffsetDateTime {
    fn ident() -> TypeIdent {
        TypeIdent::from("OffsetDateTime")
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            ident: Self::ident(),
            rs_ty: "time::OffsetDateTime".to_owned(),
            rs_dependencies: BTreeMap::from([(
                "time",
                CargoDependency {
                    version: Some("0.3"),
                    features: BTreeSet::from(["serde-well-known"]),
                    ..Default::default()
                },
            )]),
            serde_attrs: vec![r#"with = "time::serde::rfc3339""#.to_owned()],
            ts_ty: "string".to_owned(),
            ts_declaration: None,
        })
    }
}

impl Serializable for time::PrimitiveDateTime {
    fn ident() -> TypeIdent {
        TypeIdent::from("PrimitiveDateTime")
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            ident: Self::ident(),
            rs_ty: "time::PrimitiveDateTime".to_owned(),
            rs_dependencies: BTreeMap::from([(
                "time",
                CargoDependency {
                    version: Some("0.3"),
                    features: BTreeSet::from(["serde-well-known"]),
                    ..Default::default()
                },
            )]),
            serde_attrs: vec![r#"with = "time::serde::rfc3339""#.to_owned()],
            ts_ty: "string".to_owned(),
            ts_declaration: None,
        })
    }
}
