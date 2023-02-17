use crate::types::module2::Event as CoreEvent;
use fp_bindgen::prelude::Serializable;
use std::collections::BTreeMap;

#[derive(Serializable)]
pub struct Event {
    pub title: String,
    pub labels: BTreeMap<String, String>,
}

#[derive(Serializable)]
pub struct CoreEventList {
    pub events: Vec<CoreEvent>,
}
