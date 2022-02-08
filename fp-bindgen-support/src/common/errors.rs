use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use thiserror::Error;

// Note: The `Serializable` impl has to be manually kept in sync in fp-bindgen
#[derive(Debug, Serialize, Deserialize, Error)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum FPGuestError {
    /// Deserialization of data failed, possible mismatch between guest and runtime protocol version
    #[error("Failed to serde field: {path}")]
    #[serde(rename_all = "camelCase")]
    SerdeError {
        /// Path to the failed field that failed to serde
        path: String,
        message: String,
    },
    #[error("Received an invalid `FatPtr`")]
    InvalidFatPtr,
}

impl<E: Debug> From<serde_path_to_error::Error<E>> for FPGuestError {
    fn from(e: serde_path_to_error::Error<E>) -> Self {
        Self::SerdeError {
            path: e.path().to_string(),
            message: format!("{:?}", e.inner()),
        }
    }
}
