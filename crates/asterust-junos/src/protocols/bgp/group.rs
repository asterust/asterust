//! `group`
//! `protocols { bgp { group <name> { ... } } }`.
//!
//! # See also
//!
//! - [group (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/group-edit-protocols-bgp.html)

use super::dynamic_neighbor::{parse_allow, parse_dynamic_neighbor, DynamicNeighbor};
use super::neighbor::{parse_neighbor, BgpNeighbor};
use super::peer_options::{parse_peer_options, BgpPeerOptions};
use crate::{arg_of, args_of, first_word, list_values, named_children};
use asterust::config::ConfigLine;

/// A `group <name>`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpGroup {
    pub name: String,
    /// `internal` or `external`.
    pub group_type: Option<String>,
    pub description: Option<String>,
    pub peer_as: Option<String>,
    pub local_as: Option<String>,
    pub local_address: Option<String>,
    /// Route-reflector `cluster` id.
    pub cluster: Option<String>,
    /// Address families, e.g. `"inet"`, `"inet-vpn"`.
    pub families: Vec<String>,
    pub import_policies: Vec<String>,
    pub export_policies: Vec<String>,
    pub neighbors: Vec<BgpNeighbor>,
    /// Prefixes from which the group accepts sessions, written `allow [ ... ];`.
    pub allow: Vec<String>,
    /// The name of the `policy-options as-list` that a dynamic peer's AS must match.
    pub peer_as_list: Option<String>,
    pub dynamic_neighbors: Vec<DynamicNeighbor>,
    pub options: BgpPeerOptions,
}

pub(super) fn parse_group(name: &str, group: &ConfigLine) -> BgpGroup {
    let policies =
        |keyword: &str| -> Vec<String> { args_of(group, keyword).into_iter().flat_map(list_values).collect() };
    BgpGroup {
        name: name.to_string(),
        group_type: arg_of(group, "type").map(str::to_string),
        description: arg_of(group, "description").map(str::to_string),
        peer_as: arg_of(group, "peer-as").and_then(first_word).map(str::to_string),
        local_as: arg_of(group, "local-as").and_then(first_word).map(str::to_string),
        local_address: arg_of(group, "local-address").map(str::to_string),
        cluster: arg_of(group, "cluster").map(str::to_string),
        families: named_children(group, "family").map(|(f, _)| f.to_string()).collect(),
        import_policies: policies("import"),
        export_policies: policies("export"),
        neighbors: named_children(group, "neighbor")
            .map(|(address, neighbor)| parse_neighbor(address, neighbor))
            .collect(),
        allow: parse_allow(group),
        peer_as_list: arg_of(group, "peer-as-list").map(str::to_string),
        dynamic_neighbors: named_children(group, "dynamic-neighbor")
            .map(|(name, block)| parse_dynamic_neighbor(name, block))
            .collect(),
        options: parse_peer_options(group),
    }
}
