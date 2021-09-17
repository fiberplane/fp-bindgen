use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error(transparent)]
    CompileError(#[from] wasmer::CompileError),
}
