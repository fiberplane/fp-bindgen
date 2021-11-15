use super::Serializable;
use crate::types::{CargoDependency, CustomType, Type};
use std::collections::{BTreeMap, BTreeSet};

impl Serializable for http::Method {
    fn name() -> String {
        "http::Method".to_owned()
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            name: "Method".to_owned(),
            type_args: vec![],
            rs_ty: "http::Method".to_owned(),
            // rs_serde_annotations:
            rs_dependencies: BTreeMap::from([("http", CargoDependency::with_version("0.2"))]),
            ts_ty: "string".to_owned(),
            ts_declaration: Some(0),
        })
    }

    fn build_dependencies() -> BTreeSet<Type> {
        BTreeSet::new()
    }
}
