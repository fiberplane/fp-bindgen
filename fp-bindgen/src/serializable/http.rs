use super::Serializable;
use crate::types::{CargoDependency, CustomType, Type, TypeIdent};
use std::collections::{BTreeMap, BTreeSet};

impl Serializable for http::Method {
    fn ident() -> TypeIdent {
        TypeIdent::from("Method")
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            ident: Self::ident(),
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
}

impl Serializable for http::uri::Scheme {
    fn ident() -> TypeIdent {
        TypeIdent::from("Scheme")
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            ident: Self::ident(),
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
}

impl Serializable for http::uri::Uri {
    fn ident() -> TypeIdent {
        TypeIdent::from("Uri")
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            ident: Self::ident(),
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
}

impl Serializable for http::HeaderMap {
    fn ident() -> TypeIdent {
        TypeIdent::from("http::HeaderMap")
    }

    fn ty() -> Type {
        Type::Custom(CustomType {
            ident: Self::ident(),
            rs_ty: "http::HeaderMap".to_owned(),
            rs_dependencies: http_dependencies(),
            serde_attrs: vec![
                "serialize_with = \"fp_bindgen_support::http::serialize_header_map\"".to_owned(),
                "deserialize_with = \"fp_bindgen_support::http::deserialize_header_map\""
                    .to_owned(),
            ],
            ts_ty: "HeaderMap".to_owned(),
            ts_declaration: Some(r#"{ [key: string]: Uint8Array }"#.into()),
        })
    }
}

fn http_dependencies() -> BTreeMap<&'static str, CargoDependency> {
    BTreeMap::from([
        (
            "fp-bindgen-support",
            CargoDependency {
                git: Some("ssh://git@github.com/fiberplane/fp-bindgen.git"),
                branch: Some("main"),
                features: BTreeSet::from(["http"]),
                ..Default::default()
            },
        ),
        ("http", CargoDependency::with_version("0.2")),
    ])
}
