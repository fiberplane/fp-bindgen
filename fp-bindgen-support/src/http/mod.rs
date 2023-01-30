/*!
HTTP support for fp-bindgen.

Enables making HTTP requests from within the runtime.
(De)serializes various relevant data models like Methods, Headers, etc.
 */

use std::{collections::HashMap, str::FromStr};

use http::{
    header::HeaderName,
    uri::{Scheme, Uri},
    Method,
};
use serde::{
    de::{self},
    ser::SerializeMap,
    Deserialize, Deserializer, Serializer,
};
use serde_bytes::ByteBuf;

pub fn serialize_http_method<S>(method: &Method, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(method.as_str())
}

pub fn deserialize_http_method<'de, D>(deserializer: D) -> Result<Method, D::Error>
where
    D: Deserializer<'de>,
{
    MethodDef::deserialize(deserializer).map(Method::from)
}

#[non_exhaustive]
#[derive(Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum MethodDef {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Patch,
    Trace,
}

impl From<MethodDef> for Method {
    fn from(def: MethodDef) -> Method {
        match def {
            MethodDef::Get => Method::GET,
            MethodDef::Post => Method::POST,
            MethodDef::Put => Method::PUT,
            MethodDef::Delete => Method::DELETE,
            MethodDef::Head => Method::HEAD,
            MethodDef::Options => Method::OPTIONS,
            MethodDef::Connect => Method::CONNECT,
            MethodDef::Patch => Method::PATCH,
            MethodDef::Trace => Method::TRACE,
        }
    }
}

pub fn serialize_uri<S>(uri: &Uri, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&uri.to_string())
}

pub fn deserialize_uri<'de, D>(deserializer: D) -> Result<Uri, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer).and_then(|s| {
        s.parse().map_err(|_| {
            de::Error::invalid_value(
                de::Unexpected::Other("invalid url"),
                &"a string that contains a well-formatted url",
            )
        })
    })
}

pub fn serialize_uri_scheme<S>(scheme: &Scheme, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(scheme.as_str())
}

pub fn deserialize_uri_scheme<'de, D>(deserializer: D) -> Result<Scheme, D::Error>
where
    D: Deserializer<'de>,
{
    SchemeDef::deserialize(deserializer).map(Scheme::from)
}

pub fn serialize_header_map<S>(headers: &http::HeaderMap, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(None)?;

    for (key, value) in headers.clone().iter() {
        let val = serde_bytes::ByteBuf::from(value.as_bytes());
        map.serialize_entry(key.as_str(), &val)?;
    }
    map.end()
}

pub fn deserialize_header_map<'de, D>(deserializer: D) -> Result<http::HeaderMap, D::Error>
where
    D: Deserializer<'de>,
{
    let mut header_map = http::HeaderMap::new();
    let hashmap: HashMap<String, ByteBuf> = HashMap::deserialize(deserializer)?;
    for (key, value) in hashmap {
        let header_name = HeaderName::from_str(&key)
            .map_err(|e| serde::de::Error::custom(format!("Unable to parse header {key}: {e}")))?;
        header_map.insert(header_name, http::HeaderValue::from_bytes(&value).unwrap());
    }
    Ok(header_map)
}

#[non_exhaustive]
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum SchemeDef {
    Http,
    Https,
}

impl From<SchemeDef> for Scheme {
    fn from(def: SchemeDef) -> Scheme {
        match def {
            SchemeDef::Http => Scheme::HTTP,
            SchemeDef::Https => Scheme::HTTPS,
        }
    }
}
