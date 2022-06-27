use super::Body;
use fp_bindgen::prelude::Serializable;
use http::{Method, Uri};
use std::collections::HashMap;

// This example shows how HTTP requests and responses could be communicated
// while integrating the `http` crate.

/// Represents an HTTP request to be sent.
#[derive(Serializable)]
pub struct Request {
    /// The URI to submit the request to.
    pub url: Uri,

    /// HTTP method to use for the request.
    pub method: Method,

    /// HTTP headers to submit with the request.
    ///
    /// Note: We currently do not support the `Headers` type from the `http`
    ///       crate. See: <https://github.com/fiberplane/fp-bindgen/issues/102>
    pub headers: HashMap<String, String>,

    /// The body to submit with the request.
    #[fp(skip_serializing_if = "Option::is_none")]
    pub body: Option<Body>,
}

/// Represents an HTTP response we received.
///
/// Please note we currently do not support streaming responses.
#[derive(Serializable)]
pub struct Response {
    /// The response body. May be empty.
    pub body: Body,

    /// HTTP headers that were part of the response.
    ///
    /// Note: We currently do not support the `Headers` type from the `http`
    ///       crate. See: <https://github.com/fiberplane/fp-bindgen/issues/102>
    pub headers: HashMap<String, String>,

    /// HTTP status code.
    pub status_code: u16,
}

/// Represents an error that occurred while attempting to submit the request.
#[derive(Serializable)]
#[fp(tag = "type", rename_all = "snake_case")]
pub enum RequestError {
    /// Used when we know we don't have an active network connection.
    Offline,
    NoRoute,
    ConnectionRefused,
    Timeout,
    #[fp(rename_all = "snake_case")]
    ServerError {
        /// HTTP status code.
        status_code: u16,
        /// Response body.
        response: Body,
    },
    /// Misc.
    #[fp(rename = "other/misc")]
    Other {
        reason: String,
    },
}
