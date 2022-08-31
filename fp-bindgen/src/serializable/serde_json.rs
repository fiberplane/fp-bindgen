use super::Serializable;
use crate::types::{CargoDependency, CustomType, Type, TypeIdent};
use std::collections::BTreeMap;

impl Serializable for serde_json::Value {
    fn ident() -> TypeIdent {
        TypeIdent::from("Value")
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            ident: Self::ident(),
            rs_ty: "serde_json::Value".to_owned(),
            rs_dependencies: BTreeMap::from([("serde_json", CargoDependency::with_version("1.0"))]),
            serde_attrs: Vec::new(),
            ts_ty: "any".to_owned(),
            ts_declaration: None,
        })
    }
}
