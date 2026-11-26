//! `as-path-group`
//! `policy-options { as-path-group <name> { ... } }`.
//!
//! # See also
//!
//! - [as-path-group](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/as-path-group-edit-policy-options.html)

use super::as_path::{parse_as_path, AsPath};
use crate::active_children;
use asterust::config::ConfigLine;

/// An `as-path-group <name>` of AS path expressions.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AsPathGroup {
    pub name: String,
    pub as_paths: Vec<AsPath>,
}

pub(super) fn parse_as_path_group(name: &str, group: &ConfigLine) -> AsPathGroup {
    AsPathGroup {
        name: name.to_string(),
        as_paths: active_children(group).filter_map(parse_as_path).collect(),
    }
}
