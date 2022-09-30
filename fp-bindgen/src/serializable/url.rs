use std::collections::{BTreeMap, BTreeSet};
use url::Url;
use crate::{CustomType, Serializable, Type, TypeIdent};
use crate::types::CargoDependency;

impl Serializable for Url {
    fn ident() -> TypeIdent {
        TypeIdent::from("Url")
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            ident: Self::ident(),
            rs_ty: "url::Url".to_string(),
            rs_dependencies: BTreeMap::from([(
                "url",
                CargoDependency {
                    branch: None,
                    git: None,
                    path: None,
                    version: Some("2"),
                    features: BTreeSet::from(["serde"]),
                    default_features: None,
                },
            )]),
            serde_attrs: vec![],
            ts_ty: "URL".to_string(),
            ts_declaration: None
        })
    }
}
