use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Error)]
pub enum GuestError {
    /// Serialzation or deserialization of data failed, possible mismatch between guest and runtime protocol version
    ///
    #[error("Failed to serde field: {path}")]
    SerdeError {
        /// Path to the failed field that failed to serde
        path: String,
    },
}

impl<E> From<serde_path_to_error::Error<E>> for GuestError {
    fn from(e: serde_path_to_error::Error<E>) -> Self {
        Self::SerdeError {
            path: e.path().to_string(),
        }
    }
}
