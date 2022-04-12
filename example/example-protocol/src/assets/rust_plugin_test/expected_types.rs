use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, collections::HashMap, rc::Rc};

pub use example-types::ReduxAction;
pub use example-types::StateUpdate;

pub type Body = serde_bytes::ByteBuf;

/// # This is an enum with doc comments.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum DocExampleEnum {
    /// Multi-line doc comment with complex characters
    /// & " , \ ! '
    Variant1(String),
    /// Raw identifiers are supported too.
    r#Variant2 {
        /// Variant property.
        inner: i8,
    },
}

/// # This is a struct with doc comments.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DocExampleStruct {
    /// Multi-line doc comment with complex characters
    /// & " , \ ! '
    pub multi_line: String,

    /// Raw identifiers are supported too.
    pub r#type: String,
}

/// This struct is also not referenced by any function or data structure, but
/// it will show up because there is an explicit `use` statement for it in the
/// `fp_import!` macro.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExplicitedlyImportedType {
    pub you_will_see_this: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FlattenedStruct {
    pub foo: String,
    pub bar: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum FpAdjacentlyTagged {
    Foo,
    Bar(String),
    Baz { a: i8, b: u64 },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FpFlatten {
    #[serde(flatten)]
    pub flattened: FlattenedStruct,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum FpInternallyTagged {
    Foo,
    Baz { a: i8, b: u64 },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FpPropertyRenaming {
    pub foo_bar: String,
    #[serde(rename = "QUX_BAZ")]
    pub qux_baz: f64,
    pub r#raw_struct: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum FpUntagged {
    Bar(String),
    Baz { a: i8, b: u64 },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FpVariantRenaming {
    FooBar,
    #[serde(rename = "QUX_BAZ", rename_all = "SCREAMING_SNAKE_CASE")]
    QuxBaz {
        /// Will be renamed to "FOO_BAR" because of the `rename_all` on the
        /// variant.
        foo_bar: String,
        #[serde(rename = "qux_baz")]
        qux_baz: f64,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GroupImportedType1 {
    pub you_will_see_this: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GroupImportedType2 {
    pub you_will_see_this: bool,
}

pub type HttpResponse = Result<Response, RequestError>;

/// A point of an arbitrary type.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Point<T> {
    pub value: T,
}

/// Represents an HTTP request to be sent.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Request {
    /// The URI to submit the request to.
    #[serde(deserialize_with = "fp_bindgen_support::http::deserialize_uri", serialize_with = "fp_bindgen_support::http::serialize_uri")]
    pub url: http::Uri,

    /// HTTP method to use for the request.
    #[serde(deserialize_with = "fp_bindgen_support::http::deserialize_http_method", serialize_with = "fp_bindgen_support::http::serialize_http_method")]
    pub method: http::Method,

    /// HTTP headers to submit with the request.
    ///
    /// Note: We currently do not support the `Headers` type from the `http`
    ///       crate. See: https://github.com/fiberplane/fp-bindgen/issues/102
    pub headers: HashMap<String, String>,

    /// The body to submit with the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<Body>,
}

/// Represents an error that occurred while attempting to submit the request.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RequestError {
    /// Used when we know we don't have an active network connection.
    Offline,
    NoRoute,
    ConnectionRefused,
    Timeout,
    #[serde(rename_all = "snake_case")]
    ServerError {
        /// HTTP status code.
        status_code: u16,

        /// Response body.
        response: Body,
    },
    /// Misc.
    #[serde(rename = "other/misc")]
    Other { reason: String },
}

/// Represents an HTTP response we received.
///
/// Please note we currently do not support streaming responses.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Response {
    /// The response body. May be empty.
    pub body: Body,

    /// HTTP headers that were part of the response.
    ///
    /// Note: We currently do not support the `Headers` type from the `http`
    ///       crate. See: https://github.com/fiberplane/fp-bindgen/issues/102
    pub headers: HashMap<String, String>,

    /// HTTP status code.
    pub status_code: u16,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum SerdeAdjacentlyTagged {
    Foo,
    Bar(String),
    Baz { a: i8, b: u64 },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SerdeFlatten {
    #[serde(flatten)]
    pub flattened: FlattenedStruct,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum SerdeInternallyTagged {
    Foo,
    Baz { a: i8, b: u64 },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SerdePropertyRenaming {
    pub foo_bar: String,
    #[serde(rename = "QUX_BAZ")]
    pub qux_baz: f64,
    pub r#raw_struct: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum SerdeUntagged {
    Bar(String),
    Baz { a: i8, b: u64 },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SerdeVariantRenaming {
    FooBar,
    #[serde(rename = "QUX_BAZ", rename_all = "PascalCase")]
    QuxBaz {
        /// Will be renamed to "FooBar" because of the `rename_all` on the
        /// variant.
        foo_bar: String,
        #[serde(rename = "qux_baz")]
        qux_baz: f64,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct StructWithGenerics<T> {
    pub list: Vec<T>,
    pub points: Vec<Point<T>>,
    pub recursive: Vec<Point<Point<T>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub complex_nested: Option<BTreeMap<String, Vec<FloatingPoint>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optional_timestamp: Option<time::OffsetDateTime>,
}
