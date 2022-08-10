use super::Serializable;
use crate::types::{CargoDependency, CustomType, Type, TypeIdent};
use std::collections::{BTreeMap, BTreeSet};

impl Serializable for bytes::Bytes {
    fn ident() -> TypeIdent {
        TypeIdent::from("Bytes")
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            ident: Self::ident(),
            rs_ty: "bytes::Bytes".to_owned(),
            rs_dependencies: BTreeMap::from([(
                "bytes",
                CargoDependency::with_version_and_features("1", BTreeSet::from(["serde"])),
            )]),
            serde_attrs: vec![],
            ts_ty: "Uint8Array".to_owned(),
            ts_declaration: None,
        })
    }
}
