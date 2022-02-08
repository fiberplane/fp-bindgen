use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error(transparent)]
    CompileError(#[from] wasmer::CompileError),
}

#[derive(Debug, Error)]
pub enum InvocationError {
    #[error("expected function was not exported")]
    FunctionNotExported,

    #[error("returned data did not match expected type")]
    UnexpectedReturnType,

    #[error("guest returned an error: {0}")]
    GuestError(#[from] crate::common::errors::FPGuestError),

    #[error(transparent)]
    WasmerRuntimeError(#[from] wasmer::RuntimeError),
}
