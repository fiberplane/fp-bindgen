use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexGuestToHost {
    pub simple: Simple,
    pub map: BTreeMap<String, Simple>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexHostToGuest {
    pub simple: Simple,
    pub list: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Simple {
    pub foo: i32,
    pub bar: String,
}
