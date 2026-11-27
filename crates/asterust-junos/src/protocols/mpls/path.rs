//! `path`
//! `protocols { mpls { path <name> { <address> (strict | loose); ... } } }`.
//!
//! # See also
//!
//! - [path (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/path-edit-protocols-mpls.html)

use crate::{active_children, split_name};
use asterust::config::ConfigLine;

/// A `path <name>`, the explicit hops that an LSP's `primary` or `secondary` path follows. An empty path imposes no
/// routing limits.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct MplsPath {
    pub name: String,
    pub hops: Vec<MplsPathHop>,
}

/// A hop of an [`MplsPath`], in order.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct MplsPathHop {
    /// An IP address, a hostname or the name of an abstract hop.
    pub address: String,
    /// `strict` or `loose`, or for an abstract hop `abstract`, `loose` or `loose-link`.
    pub qualifier: Option<String>,
}

pub(super) fn parse_path(name: &str, path: &ConfigLine) -> MplsPath {
    MplsPath {
        name: name.to_string(),
        hops: active_children(path)
            .map(|hop| {
                let (address, rest) = split_name(&hop.text);
                MplsPathHop {
                    address: address.to_string(),
                    qualifier: Some(rest).filter(|q| !q.is_empty()).map(str::to_string),
                }
            })
            .collect(),
    }
}
