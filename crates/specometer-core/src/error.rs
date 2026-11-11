use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpecometerError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
