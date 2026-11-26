//! `neighbor`
//! `protocols { bgp { group <name> { neighbor <address> { ... } } } }`.
//!
//! # See also
//!
//! - [neighbor (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/neighbor-edit-protocols-bgp.html)

use super::peer_options::{parse_peer_options, BgpPeerOptions};
use crate::{arg_of, first_word};
use asterust::config::ConfigLine;

/// A `neighbor <address>` in a BGP group. Settings inherited from the group are not resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpNeighbor {
    pub address: String,
    pub description: Option<String>,
    pub peer_as: Option<String>,
    pub local_address: Option<String>,
    pub options: BgpPeerOptions,
}

pub(super) fn parse_neighbor(address: &str, neighbor: &ConfigLine) -> BgpNeighbor {
    BgpNeighbor {
        address: address.to_string(),
        description: arg_of(neighbor, "description").map(str::to_string),
        peer_as: arg_of(neighbor, "peer-as").and_then(first_word).map(str::to_string),
        local_address: arg_of(neighbor, "local-address").map(str::to_string),
        options: parse_peer_options(neighbor),
    }
}
