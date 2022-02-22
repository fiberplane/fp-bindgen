use super::Serializable;
use crate::types::{CargoDependency, CustomType, Type, TypeIdent};
use std::collections::{BTreeMap, BTreeSet};

impl Serializable for rmpv::Value {
    fn ident() -> TypeIdent {
        TypeIdent::from("Value")
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            ident: Self::ident(),
            rs_ty: "rmpv::Value".to_owned(),
            rs_dependencies: BTreeMap::from([(
                "rmpv",
                CargoDependency {
                    version: Some("1.0.0"),
                    features: BTreeSet::from(["with-serde"]),
                    git: None,
                    branch: None,
                    path: None,
                },
            )]),
            serde_attrs: Vec::new(),
            ts_ty: "string".to_owned(),
            ts_declaration: None,
        })
    }
}
