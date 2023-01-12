use fp_bindgen::prelude::Serializable;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize, Serializable)]
#[fp(rename_all = "camelCase")]
pub struct StructWithOptions {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub potentially_optional_string: String,
}
