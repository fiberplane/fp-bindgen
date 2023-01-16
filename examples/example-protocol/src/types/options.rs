use fp_bindgen::prelude::Serializable;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize, Serializable)]
#[fp(rename_all = "camelCase")]
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
