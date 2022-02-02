use super::Serializable;
use crate::types::{CargoDependency, CustomType, Type, TypeIdent};
use std::collections::BTreeMap;

impl Serializable for serde_bytes::ByteBuf {
    fn ident() -> TypeIdent {
        TypeIdent::from("ByteBuf".to_owned())
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            ident: Self::ident(),
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
}
