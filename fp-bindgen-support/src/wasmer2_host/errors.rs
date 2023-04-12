use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error(transparent)]
    CompileError(#[from] wasmer::CompileError),
}

#[derive(Debug, Error)]
pub enum InvocationError {
    #[error("expected function was not exported: {0}")]
    FunctionNotExported(String),

    #[error("returned data did not match expected type")]
    UnexpectedReturnType,

    #[error(transparent)]
    WasmerRuntimeError(#[from] wasmer::RuntimeError),
}
