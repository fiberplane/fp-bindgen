use http::Method;
use serde::{Deserialize, Deserializer, Serializer};

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
