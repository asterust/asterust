//! `interfaces { <name> { ether-options { 802.3ad { ... } } } }`.

use crate::{active_children, arg_of, child_block, statement_words, value_after};
use asterust::config::ConfigLine;

/// The `802.3ad` statement that makes an interface a member of an aggregated Ethernet bundle.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Ieee8023ad {
    /// The bundle, e.g. `"ae0"`.
    pub aggregated_interface: Option<String>,
    /// `primary` or `backup`, for link protection.
    pub role: Option<String>,
    pub link_index: Option<String>,
    pub distribution_list: Option<String>,
    /// From `lacp { port-priority ...; }`.
    pub lacp_port_priority: Option<String>,
}

/// The `802.3ad` statement of `options`, written `802.3ad ae0;`, `802.3ad ae0 { ... }` or `802.3ad { ae0; ... }`.
pub(super) fn parse_ieee_802_3ad(options: &ConfigLine) -> Option<Ieee8023ad> {
    let statement = active_children(options).find(|c| c.text == "802.3ad" || c.arg("802.3ad").is_some())?;
    let words = statement_words(options, "802.3ad")?;
    Some(Ieee8023ad {
        aggregated_interface: words.iter().find(|w| w.starts_with("ae")).map(|w| w.to_string()),
        role: words
            .iter()
            .find(|w| matches!(**w, "primary" | "backup"))
            .map(|w| w.to_string()),
        link_index: value_after(&words, "link-index"),
        distribution_list: value_after(&words, "distribution-list"),
        lacp_port_priority: child_block(statement, "lacp")
            .and_then(|lacp| arg_of(lacp, "port-priority"))
            .map(str::to_string),
    })
}
