//! `segment-list`
//! `protocols { source-packet-routing { segment-list <name> { <hop> { ... } } } }`.
//!
//! # See also
//!
//! - [segment-list](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/segment-list-protocols-source-packet-routing.html)
//! - [dynamic (Source Packet Routing)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/dynamic-edit-protocols-source-packet-routing-segment-list.html)

use crate::{active_children, has_leaf, value_after};
use asterust::config::ConfigLine;

/// The statements of a segment list that are not hops.
const OPTIONS: [&str; 5] = ["srv6", "compute", "dynamic", "inherit-label-nexthops", "auto-translate"];

/// A `segment-list <name>`. The contents of `dynamic` are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SegmentList {
    pub name: String,
    /// An SRv6-TE segment list, whose hops are `srv6-sid`s.
    pub srv6: bool,
    pub compute: bool,
    pub dynamic: bool,
    pub inherit_label_nexthops: bool,
    /// An `auto-translate` block is configured, which allows IP addresses instead of labels.
    pub auto_translate: bool,
    pub hops: Vec<SegmentHop>,
}

/// A hop of a [`SegmentList`], written `<name> srv6-sid <sid>;` or as a block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SegmentHop {
    pub name: String,
    pub strict: bool,
    pub loose: bool,
    pub ip_address: Option<String>,
    pub label: Option<String>,
    /// `node`, for a hop that is the address of a node rather than a link.
    pub label_type: Option<String>,
    pub srv6_sid: Option<String>,
}

pub(super) fn parse_segment_list(name: &str, list: &ConfigLine) -> SegmentList {
    SegmentList {
        name: name.to_string(),
        srv6: has_leaf(list, "srv6"),
        compute: has_leaf(list, "compute"),
        dynamic: statement_present(list, "dynamic"),
        inherit_label_nexthops: has_leaf(list, "inherit-label-nexthops"),
        auto_translate: statement_present(list, "auto-translate"),
        hops: active_children(list)
            .filter(|child| {
                child
                    .text
                    .split_whitespace()
                    .next()
                    .is_some_and(|first| !OPTIONS.contains(&first))
            })
            .map(parse_hop)
            .collect(),
    }
}

fn statement_present(list: &ConfigLine, keyword: &str) -> bool {
    active_children(list).any(|child| child.text == keyword || child.arg(keyword).is_some())
}

fn parse_hop(hop: &ConfigLine) -> SegmentHop {
    let mut words = hop.text.split_whitespace();
    let name = words.next().unwrap_or_default().to_string();
    let words: Vec<&str> = words
        .chain(active_children(hop).flat_map(|child| child.text.split_whitespace()))
        .collect();
    SegmentHop {
        name,
        strict: words.contains(&"strict"),
        loose: words.contains(&"loose"),
        ip_address: value_after(&words, "ip-address"),
        label: value_after(&words, "label"),
        label_type: value_after(&words, "label-type"),
        srv6_sid: value_after(&words, "srv6-sid"),
    }
}
