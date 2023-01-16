#![allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, rc::Rc};

pub use redux_example::ReduxAction;
pub use redux_example::StateUpdate;

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

/// A point of an arbitrary type, with explicit trait bounds.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExplicitBoundPoint<T: std::fmt::Debug + std::fmt::Display> {
    pub value: T,
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

pub type FloatingPoint = Point<f64>;

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

pub type HttpResult = Result<Response, RequestError>;

pub type Int64 = u64;

/// Our struct for passing date time instances.
///
/// We wrap the `OffsetDateTime` type in a new struct so that the Serde
/// attributes can be inserted. These are necessary to enable RFC3339
/// formatting. Without a wrapper type like this, we would not be able to pass
/// date time instances directly to function arguments and we might run into
/// trouble embedding them into certain generic types.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MyDateTime(
    #[serde(with = "time::serde::rfc3339")]
    pub time::OffsetDateTime,
);

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
    #[serde(deserialize_with = "fp_bindgen_support::http::deserialize_header_map", serialize_with = "fp_bindgen_support::http::serialize_header_map")]
    pub headers: http::HeaderMap,

    /// The body to submit with the request.
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(deserialize_with = "fp_bindgen_support::http::deserialize_header_map", serialize_with = "fp_bindgen_support::http::serialize_header_map")]
    pub headers: http::HeaderMap,

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
    pub complex_nested: Option<BTreeMap<String, Vec<FloatingPoint>>>,
    pub optional_timestamp: Option<MyDateTime>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructWithOptions {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub filled_string: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub empty_string: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filled_option_string: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empty_option_string: Option<String>,
    #[serde(default)]
    pub never_skipped_filled_option_string: Option<String>,
    #[serde(default)]
    pub never_skipped_empty_option_string: Option<String>,
}
