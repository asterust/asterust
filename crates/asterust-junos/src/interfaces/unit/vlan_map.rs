//! Extracts the `input-vlan-map` and `output-vlan-map` of a logical unit.
//!
//! Configuration path: `interfaces { <name> { unit <n> { input-vlan-map { ... } } } }`.
//!
//! # See also
//!
//! - [input-vlan-map](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/input-vlan-map-edit-interfaces-ge.html)
//! - [output-vlan-map](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/output-vlan-map-edit-interfaces-ge.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

const OPERATIONS: [&str; 8] = [
    "pop",
    "pop-pop",
    "pop-swap",
    "push",
    "push-push",
    "swap",
    "swap-push",
    "swap-swap",
];

/// An `input-vlan-map` or `output-vlan-map`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct VlanMap {
    /// `pop`, `pop-pop`, `pop-swap`, `push`, `push-push`, `swap`, `swap-push` or `swap-swap`.
    pub operation: Option<String>,
    pub tag_protocol_id: Option<String>,
    pub vlan_id: Option<String>,
    pub inner_tag_protocol_id: Option<String>,
    pub inner_vlan_id: Option<String>,
}

/// The `keyword` map of `unit`, written as a block or on one line (`output-vlan-map pop;`).
pub(super) fn parse_vlan_map(unit: &ConfigLine, keyword: &str) -> Option<VlanMap> {
    let words = statement_words(unit, keyword)?;
    Some(VlanMap {
        operation: words.iter().find(|w| OPERATIONS.contains(w)).map(|w| w.to_string()),
        tag_protocol_id: value_after(&words, "tag-protocol-id"),
        vlan_id: value_after(&words, "vlan-id"),
        inner_tag_protocol_id: value_after(&words, "inner-tag-protocol-id"),
        inner_vlan_id: value_after(&words, "inner-vlan-id"),
    })
}
