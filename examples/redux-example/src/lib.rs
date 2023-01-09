use fp_bindgen::prelude::Serializable;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

/// Example for representing Redux actions.
#[derive(Serializable, Serialize, Deserialize)]
#[fp(rust_module = "redux_example")]
#[serde(rename_all = "snake_case", tag = "type", content = "payload")]
pub enum ReduxAction {
    ClearTitle,
    UpdateTitle { title: String },
}

/// Example for how Redux state could be stored in Rust.
///
/// Any fields that do not implement `Copy` are wrapped in `Rc` so that we can
/// cheaply clone the state, as well as to cheaply perform a diff between the
/// old and new state. This is done in `StateUpdate::from_states()`.
#[derive(Clone, Default, Serializable, Serialize, Deserialize)]
#[fp(rust_module = "redux_example")]
#[serde(rename_all = "camelCase")]
pub struct ReduxState {
    pub title: Rc<String>,
    pub revision: u16,
}

/// A state update to communicate to the Redux host.
///
/// Fields are wrapped in `Option`. If any field is `None` it means it hasn't
/// changed.
#[derive(Serializable, Serialize, Deserialize)]
#[fp(rust_module = "redux_example")]
#[serde(rename_all = "camelCase")]
pub struct StateUpdate {
    pub title: Option<Rc<String>>,
    pub revision: Option<u16>,
}

impl StateUpdate {
    /// Creates a state update that includes all the fields from the new state
    /// that have changed from the old state.
    pub fn from_state(old: &ReduxState, new: &ReduxState) -> Self {
        Self {
            title: if Rc::ptr_eq(&old.title, &new.title) {
                None
            } else {
                Some(new.title.clone())
            },
            revision: if old.revision == new.revision {
                None
            } else {
                Some(new.revision)
            },
        }
    }
}
