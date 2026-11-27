//! `source-routing-path`
//! `protocols { source-packet-routing { source-routing-path <name> { primary ...; } } }`.
//!
//! # See also
//!
//! - [source-routing-path](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/autogen-protocols-source-packet-routing-source-routing-path.html)

use crate::{active_children, arg_of, has_leaf, value_after};
use asterust::config::ConfigLine;

/// A `source-routing-path <name>`, a static SR-TE policy. The `compute` blocks of the segment lists are read as their
/// profile name only.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SourceRoutingPath {
    pub name: String,
    /// The tunnel type is SRv6.
    pub srv6: bool,
    /// The tunnel endpoint.
    pub to: Option<String>,
    /// The tunnel source address. It is not in the statement's syntax, but the SRv6-TE example sets it.
    pub from: Option<String>,
    pub color: Option<String>,
    pub binding_sid: Option<String>,
    pub metric: Option<String>,
    pub no_ingress: bool,
    pub sr_preference: Option<String>,
    pub te_group_id: Option<String>,
    pub lsp_external_controller: Option<String>,
    pub primary: Vec<PathSegmentList>,
    pub secondary: Vec<PathSegmentList>,
}

/// A segment list used by the `primary` or `secondary` paths of a [`SourceRoutingPath`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PathSegmentList {
    /// The name of the `segment-list`, or of the path.
    pub name: String,
    pub weight: Option<String>,
    pub return_path: Option<String>,
    /// The compute profile.
    pub compute: Option<String>,
}

pub(super) fn parse_source_routing_path(name: &str, path: &ConfigLine) -> SourceRoutingPath {
    let field = |keyword: &str| arg_of(path, keyword).map(str::to_string);
    SourceRoutingPath {
        name: name.to_string(),
        srv6: has_leaf(path, "srv6"),
        to: field("to"),
        from: field("from"),
        color: field("color"),
        binding_sid: field("binding-sid"),
        metric: field("metric"),
        no_ingress: has_leaf(path, "no-ingress"),
        sr_preference: field("sr-preference"),
        te_group_id: field("te-group-id"),
        lsp_external_controller: field("lsp-external-controller"),
        primary: parse_segment_lists(path, "primary"),
        secondary: parse_segment_lists(path, "secondary"),
    }
}

/// Reads `primary <name> { weight ...; }`, `primary <name> weight ...;` and `primary { <name> weight ...; }`.
fn parse_segment_lists(path: &ConfigLine, keyword: &str) -> Vec<PathSegmentList> {
    let mut lists = Vec::new();
    for statement in active_children(path).filter(|c| c.text == keyword || c.arg(keyword).is_some()) {
        match statement.arg(keyword) {
            Some(inline) => lists.push(parse_entry(inline, statement)),
            None => lists.extend(active_children(statement).map(|entry| parse_entry(&entry.text, entry))),
        }
    }
    lists
}

/// `text` is the name and any options on the same line; `node` supplies the options written as children.
fn parse_entry(text: &str, node: &ConfigLine) -> PathSegmentList {
    let mut words = text.split_whitespace();
    let name = words.next().unwrap_or_default().to_string();
    let words: Vec<&str> = words
        .chain(active_children(node).flat_map(|child| child.text.split_whitespace()))
        .collect();
    PathSegmentList {
        name,
        weight: value_after(&words, "weight"),
        return_path: value_after(&words, "return-path"),
        compute: value_after(&words, "compute"),
    }
}
