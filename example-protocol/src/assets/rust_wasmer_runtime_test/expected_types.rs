use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, collections::HashMap};

pub use example_bindings::RequestMethod;
pub use example_bindings::RequestOptions;
pub use example_bindings::Response;

pub type Body = serde_bytes::ByteBuf;

pub type ComplexAlias = ComplexGuestToHost;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexGuestToHost {
    pub simple: Simple,
    pub map: BTreeMap<String, Simple>,
    pub timestamp: time::OffsetDateTime,
}

/// Multi-line doc comment with complex characters
/// & " , \ ! '
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexHostToGuest {
    pub simple: Simple,
    pub list: Vec<f64>,
    pub points: Vec<Point<f64>>,
    pub recursive: Vec<Point<Point<f64>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub complex_nested: Option<BTreeMap<String, Vec<Point<f64>>>>,
    pub timestamp: time::OffsetDateTime,
    #[serde(default, rename = "optional_timestamp", skip_serializing_if = "Option::is_none")]
    pub renamed: Option<time::OffsetDateTime>,

    /// Raw identifiers are supported too.
    pub r#type: String,
    pub id: uuid::Uuid,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplicitedlyImportedType {
    pub you_will_see_this: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupImportedType1 {
    pub you_will_see_this: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupImportedType2 {
    pub you_will_see_this: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Point<T> {
    pub value: T,
}

/// Represents an error with the request.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RequestError {
    /// Used when we know we don't have an active network connection.
    Offline,
    NoRoute,
    ConnectionRefused,
    Timeout,
    #[serde(rename_all = "camelCase")]
    ServerError {
        /// HTTP status code.
        status_code: u16,

        /// Response body.
        response: Body,
    },
    /// Misc.
    #[serde(rename = "other/misc", rename_all = "camelCase")]
    Other { reason: String },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Simple {
    pub foo: i32,
    pub bar: String,
}
