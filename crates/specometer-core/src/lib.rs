#![forbid(unsafe_code)]

pub mod error;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use error::SpecometerError;
