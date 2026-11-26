//! Parses vendor-style hierarchical configuration text into a generic tree.
//!
//! Pure Rust, no Python/PyO3 dependency, so any extension module (or
//! non-Python tool) can depend on it directly and add its own bindings.
//!
//! - [`config`]: the vendor-agnostic tree types and one parser per syntax family.
//! - [`device`]: the traits that platform crates implement to share getters.
//!
//! # Examples
//!
//! ```
//! use asterust::config::cisco;
//!
//! let config = cisco::parse("router ospf 1\n router-id 10.0.0.1\n!\n");
//! let ospf = config.blocks[0].get_root_line();
//! assert_eq!(ospf.child_arg("router-id"), Some("10.0.0.1"));
//! ```

pub mod config;
pub mod device;
