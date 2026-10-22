//! asterust: reusable parsing primitives for network device CLI output.
//!
//! Pure Rust, no Python/PyO3 dependency, so any extension module (or
//! non-Python tool) can depend on it directly and add its own bindings.

pub mod clean;
pub mod config;
pub mod date;
pub mod paths;
pub mod prompt;
