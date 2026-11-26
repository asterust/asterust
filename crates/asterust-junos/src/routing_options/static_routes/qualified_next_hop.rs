//! `qualified-next-hop`
//! `routing-options { static { route <prefix> { qualified-next-hop <next-hop> { ... } } } }`.
//!
//! # See also
//!
//! - [qualified-next-hop (Static Routes)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/qualified-next-hop-edit-routing-options.html)

use crate::{split_name, value_after};
use asterust::config::ConfigLine;

/// A `qualified-next-hop` of a static route: a next hop with settings that apply to it alone. The
/// `bfd-liveness-detection` block is not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct QualifiedNextHop {
    /// An address, or an interface name for an unnumbered or point-to-point interface.
    pub next_hop: String,
    pub interface: Option<String>,
    pub mac_address: Option<String>,
    pub preference: Option<String>,
    pub metric: Option<String>,
    pub tag: Option<String>,
    pub order: Option<String>,
}

/// Reads `<next-hop> [<keyword> <value> ...]`, with the settings written inline or as child statements.
pub(super) fn parse_qualified_next_hop(arg: &str, children: &[&ConfigLine]) -> QualifiedNextHop {
    let (next_hop, inline) = split_name(arg);
    let words: Vec<&str> = inline
        .split_whitespace()
        .chain(children.iter().flat_map(|child| child.text.split_whitespace()))
        .collect();
    QualifiedNextHop {
        next_hop: next_hop.to_string(),
        interface: value_after(&words, "interface"),
        mac_address: value_after(&words, "mac-address"),
        preference: value_after(&words, "preference"),
        metric: value_after(&words, "metric"),
        tag: value_after(&words, "tag"),
        order: value_after(&words, "order"),
    }
}
