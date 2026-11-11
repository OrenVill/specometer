#![forbid(unsafe_code)]

pub mod error;
pub mod line_span;
pub mod spec_index;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use error::SpecometerError;
