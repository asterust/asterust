//! `route`
//! `routing-options { static { route <prefix> { ... } } }`.
//!
//! # See also
//!
//! - [static (Routing Options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/static-edit-routing-options.html)
//! - [discard (routing-options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/discard-edit-routing-options.html)

use super::qualified_next_hop::{parse_qualified_next_hop, QualifiedNextHop};
use crate::routing_options::route_attributes::{parse_route_attributes, RouteAttributes};
use crate::{active_children, list_values, split_keyword, split_name, unquote};
use asterust::config::ConfigLine;

/// A static `route <prefix>`. The next hop is one of `next_hops`, `discard`, `reject`, `receive`, `next_table` or
/// `qualified_next_hops`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct StaticRoute {
    pub prefix: String,
    pub next_hops: Vec<String>,
    pub discard: bool,
    pub reject: bool,
    pub receive: bool,
    pub next_table: Option<String>,
    pub qualified_next_hops: Vec<QualifiedNextHop>,
    pub attributes: RouteAttributes,
}

/// Reads a route written on one line (`route 10.0.0.0/8 discard;`) or as a block.
pub(super) fn parse_route(arg: &str, line: &ConfigLine) -> StaticRoute {
    let (prefix, inline) = split_name(arg);
    let children: Vec<&ConfigLine> = active_children(line).collect();

    // `route <prefix> qualified-next-hop <next-hop> { ... }` keeps the settings in the route's own block.
    let mut statements: Vec<(&str, Vec<&ConfigLine>)> = Vec::new();
    if inline.starts_with("qualified-next-hop") {
        statements.push((inline, children));
    } else {
        if !inline.is_empty() {
            statements.push((inline, Vec::new()));
        }
        statements.extend(
            children
                .into_iter()
                .map(|child| (child.text.as_str(), active_children(child).collect())),
        );
    }

    let mut route = StaticRoute {
        prefix: prefix.to_string(),
        next_hops: Vec::new(),
        discard: false,
        reject: false,
        receive: false,
        next_table: None,
        qualified_next_hops: Vec::new(),
        attributes: RouteAttributes::default(),
    };
    let mut attribute_statements = Vec::new();
    for (text, children) in &statements {
        let (keyword, rest) = split_keyword(text);
        match keyword {
            "next-hop" => route.next_hops.extend(list_values(rest)),
            "discard" => route.discard = true,
            "reject" => route.reject = true,
            "receive" => route.receive = true,
            "next-table" => route.next_table = Some(unquote(rest).to_string()),
            "qualified-next-hop" => route.qualified_next_hops.push(parse_qualified_next_hop(rest, children)),
            _ => attribute_statements.push(*text),
        }
    }
    route.attributes = parse_route_attributes(attribute_statements);
    route
}
