//! Extracts the `prefix-list` entries of `policy-options`.
//!
//! Configuration path: `policy-options { prefix-list <name> { ... } }`.
//!
//! # See also
//!
//! - [prefix-list](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/prefix-list-edit-policy-options.html)

use crate::{active_children, unquote};
use asterust::config::ConfigLine;

/// A `prefix-list <name>`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PrefixList {
    pub name: String,
    pub prefixes: Vec<String>,
    pub apply_path: Option<String>,
}

pub(super) fn parse_prefix_list(name: &str, line: &ConfigLine) -> PrefixList {
    let mut list = PrefixList {
        name: name.to_string(),
        prefixes: Vec::new(),
        apply_path: None,
    };
    for child in active_children(line) {
        if let Some(path) = child.arg("apply-path") {
            list.apply_path = Some(unquote(path).to_string());
        } else if !child.text.contains(' ') {
            // Prefixes are the only statements without arguments; `apply-groups` and the like have a space.
            list.prefixes.push(child.text.clone());
        }
    }
    list
}
