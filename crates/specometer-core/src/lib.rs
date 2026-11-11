#![forbid(unsafe_code)]

pub mod bundle;
pub mod closure;
pub mod error;
pub mod line_span;
pub mod plan_index;
pub mod pipeline;
pub mod resolver;
pub mod spec_index;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use error::SpecometerError;
