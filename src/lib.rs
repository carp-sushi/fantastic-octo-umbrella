pub mod v1;

pub mod config;
pub mod health;
pub mod validate;

/// Project level error type
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid argument: {message}")]
    InvalidArgument { message: String },
    #[error("internal error: {message}")]
    InternalError { message: String },
    #[error("not found error: {message}")]
    NotFoundError { message: String },
}

/// Project level result type
pub type Result<T> = std::result::Result<T, Error>;
