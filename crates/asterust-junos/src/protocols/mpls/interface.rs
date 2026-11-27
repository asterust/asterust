//! `interface`
//! `protocols { mpls { interface (<name> | all) { ... } } }`.
//!
//! # See also
//!
//! - [interface (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/interface-edit-protocols-mpls.html)

use crate::{args_of, has_leaf, list_values};
use asterust::config::ConfigLine;

/// An `interface <name>` entry of MPLS; the name can be `all`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct MplsInterface {
    pub name: String,
    pub disable: bool,
    /// The `admin-group` names applied to the link.
    pub admin_groups: Vec<String>,
    pub srlg: Vec<String>,
}

pub(super) fn parse_interface(name: &str, interface: &ConfigLine) -> MplsInterface {
    let list =
        |keyword: &str| -> Vec<String> { args_of(interface, keyword).into_iter().flat_map(list_values).collect() };
    MplsInterface {
        name: name.to_string(),
        disable: has_leaf(interface, "disable"),
        admin_groups: list("admin-group"),
        srlg: list("srlg"),
    }
}
