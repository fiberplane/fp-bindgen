use super::Serializable;
use crate::types::{CargoDependency, CustomType, Type};
use std::collections::{BTreeMap, BTreeSet};

impl Serializable for http::Method {
    fn name() -> String {
        "Method".to_owned()
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            name: "Method".to_owned(),
            type_args: vec![],
            rs_ty: "http::Method".to_owned(),
            rs_dependencies: http_dependencies(),
            serde_attrs: vec![
                "serialize_with = \"fp_bindgen_support::http::serialize_http_method\"".to_owned(),
                "deserialize_with = \"fp_bindgen_support::http::deserialize_http_method\""
                    .to_owned(),
            ],
            ts_ty: "Method".to_owned(),
            ts_declaration: Some(
                r#"
    | "GET"
    | "POST"
    | "PUT"
    | "DELETE"
    | "HEAD"
    | "OPTIONS"
    | "CONNECT"
    | "PATCH"
    | "TRACE""#
                    .to_owned(),
            ),
        })
    }

    fn build_dependencies() -> BTreeSet<Type> {
        BTreeSet::new()
    }
}

impl Serializable for http::uri::Scheme {
    fn name() -> String {
        "Scheme".to_owned()
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            name: "Scheme".to_owned(),
            type_args: vec![],
            rs_ty: "http::Scheme".to_owned(),
            rs_dependencies: http_dependencies(),
            serde_attrs: vec![
                "serialize_with = \"fp_bindgen_support::http::serialize_uri_scheme\"".to_owned(),
                "deserialize_with = \"fp_bindgen_support::http::deserialize_uri_scheme\""
                    .to_owned(),
            ],
            ts_ty: "Scheme".to_owned(),
            ts_declaration: Some(r#""http" | "https""#.to_owned()),
        })
    }

    fn build_dependencies() -> BTreeSet<Type> {
        BTreeSet::new()
    }
}

impl Serializable for http::uri::Uri {
    fn name() -> String {
        "Uri".to_owned()
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            name: "Uri".to_owned(),
            type_args: vec![],
            rs_ty: "http::Uri".to_owned(),
            rs_dependencies: http_dependencies(),
            serde_attrs: vec![
                "serialize_with = \"fp_bindgen_support::http::serialize_uri\"".to_owned(),
                "deserialize_with = \"fp_bindgen_support::http::deserialize_uri\"".to_owned(),
            ],
            ts_ty: "string".to_owned(),
            ts_declaration: None,
        })
    }

    fn build_dependencies() -> BTreeSet<Type> {
        BTreeSet::new()
    }
}

fn http_dependencies() -> BTreeMap<&'static str, CargoDependency> {
    BTreeMap::from([
        (
            "fp-bindgen-support",
            CargoDependency {
                git: Some("ssh://git@github.com/fiberplane/fp-bindgen.git"),
                branch: Some("main"),
                path: None,
                version: None,
                features: BTreeSet::from(["http"]),
            },
        ),
        ("http", CargoDependency::with_version("0.2")),
    ])
}
