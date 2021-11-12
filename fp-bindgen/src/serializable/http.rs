use super::Serializable;
use crate::types::{CustomType, Type};
use std::collections::{BTreeMap, BTreeSet};

impl<T: Serializable> Serializable for http::Response<T> {
    fn name() -> String {
        "http::Response".to_owned()
    }

    fn ty() -> Type {
        let t = T::ty();
        let rs_ty = format!(
            "http::Response<{}>",
            match &t {
                Type::Custom(custom) => custom.rs_ty,
                other => other.name(),
            }
        );
        let ts_ty = format!(
            "http::Response<{}>",
            match &t {
                Type::Custom(custom) => custom.ts_ty,
                other => other.name(),
            }
        );

        Type::Custom(CustomType {
            name: format!("Response<{}>", t.name()),
            type_args: vec![t],
            rs_ty, // TODO: Add `remote =` serde annotation! (https://serde.rs/remote-derive.html)
            rs_dependencies: BTreeMap::from([("http".to_owned(), r#""0.2""#.to_owned())]),
            ts_ty,
            ts_declaration: Some(),
        })
    }

    fn build_dependencies() -> BTreeSet<Type> {
        T::type_with_dependencies()
    }
}
