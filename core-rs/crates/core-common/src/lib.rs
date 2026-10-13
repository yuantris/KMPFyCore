use thiserror::Error;
pub type CoreResult<T> = Result<T, CoreError>;
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid argument: {0}")] InvalidArgument(String),
    #[error("parse error: {0}")] Parse(String),
    #[error("I/O error: {0}")] Io(#[from] std::io::Error),
    #[error("archive error: {0}")] Archive(String),
    #[error("internal error: {0}")] Internal(String),
}
