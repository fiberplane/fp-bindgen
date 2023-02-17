use fp_bindgen::prelude::Serializable;

#[derive(Serializable)]
pub struct Event {
    pub severity: i32,
}
