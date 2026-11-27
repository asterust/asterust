//! `resolution-map`
//! `policy-options { resolution-map <name> { mode <mode>; } }`.
//!
//! # See also
//!
//! - [resolution-map](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/resolution-map-edit-policy-options.html)

use crate::{args_of, list_values};
use asterust::config::ConfigLine;

/// A `resolution-map <name>`, the protocol next hop resolution modes that a `then resolution-map` action applies.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResolutionMap {
    pub name: String,
    /// `color-only` and `ip-color`, in the order they are tried.
    pub modes: Vec<String>,
}

pub(super) fn parse_resolution_map(name: &str, line: &ConfigLine) -> ResolutionMap {
    ResolutionMap {
        name: name.to_string(),
        modes: args_of(line, "mode").into_iter().flat_map(list_values).collect(),
    }
}
