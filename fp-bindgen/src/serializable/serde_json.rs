use super::Serializable;
use crate::prelude::TypeMap;
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

impl<K, V> Serializable for serde_json::Map<K, V>
where
    K: Serializable,
    V: Serializable,
{
    fn ident() -> TypeIdent {
        TypeIdent {
            name: "serde_json::Map".to_string(),
            generic_args: vec![
                (TypeIdent::from("K"), vec![]),
                (TypeIdent::from("V"), vec![]),
            ],
            ..Default::default()
        }
    }

    fn ty() -> Type {
        Type::Map(
            "serde_json::Map".to_owned(),
            TypeIdent::from("K"),
            TypeIdent::from("V"),
        )
    }

    fn collect_types(types: &mut TypeMap) {
        types.entry(Self::ident()).or_insert_with(Self::ty);
        K::collect_types(types);
        V::collect_types(types);
    }
}
