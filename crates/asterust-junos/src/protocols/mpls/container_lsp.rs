//! `container-label-switched-path`
//! `protocols { mpls { container-label-switched-path <name> { to <address>; ... } } }`.
//!
//! # See also
//!
//! - [container-label-switched-path](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/container-label-switched-path-edit-protocols-mpls.html)

use crate::{active_children, arg_of, has_leaf};
use asterust::config::ConfigLine;

/// A `container-label-switched-path <name>`, a tunnel made of several member LSPs to the same egress router.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct MplsContainerLsp {
    pub name: String,
    pub to: Option<String>,
    pub description: Option<String>,
    pub disable: bool,
    /// Added to the names of the member LSPs.
    pub suffix: Option<String>,
    pub label_switched_path_template: Option<String>,
    /// A `splitting-merging` statement is configured; its options are not extracted.
    pub splitting_merging: bool,
}

pub(super) fn parse_container_lsp(name: &str, lsp: &ConfigLine) -> MplsContainerLsp {
    let field = |keyword: &str| arg_of(lsp, keyword).map(str::to_string);
    MplsContainerLsp {
        name: name.to_string(),
        to: field("to"),
        description: field("description"),
        disable: has_leaf(lsp, "disable"),
        suffix: field("suffix"),
        label_switched_path_template: field("label-switched-path-template"),
        splitting_merging: active_children(lsp)
            .any(|c| c.text == "splitting-merging" || c.arg("splitting-merging").is_some()),
    }
}
