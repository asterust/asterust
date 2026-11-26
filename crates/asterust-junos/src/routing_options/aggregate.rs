//! `aggregate`
//! `routing-options { aggregate { route <prefix> { ... } } }`.
//!
//! # See also
//!
//! - [aggregate (Routing)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/aggregate-edit-routing-options.html)
//! - [policy (Aggregate and Generated Routes)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/policy-edit-routing-options.html)
//! - [brief](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/brief-edit-routing-options.html)
//! - [full](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/full-edit-routing-options.html)
//! - [discard (routing-options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/discard-edit-routing-options.html)

use super::route_attributes::{parse_route_attributes, route_statements, RouteAttributes};
use crate::{active_children, child_block, split_keyword, split_name, unquote};
use asterust::config::ConfigLine;

/// An aggregate `route <prefix>`. `as-path` is not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AggregateRoute {
    pub prefix: String,
    /// The policy that selects the contributing routes.
    pub policy: Option<String>,
    pub brief: bool,
    pub full: bool,
    pub discard: bool,
    pub next_table: Option<String>,
    pub algorithm: Option<String>,
    pub attributes: RouteAttributes,
}

/// The `aggregate` routes of a routing table.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AggregateRoutes {
    /// The `defaults` attributes; `policy`, `brief`, `full` and `discard` inside `defaults` are not extracted.
    pub defaults: Option<RouteAttributes>,
    pub routes: Vec<AggregateRoute>,
}

pub(super) fn parse_aggregate_routes(aggregate: &ConfigLine) -> AggregateRoutes {
    AggregateRoutes {
        defaults: child_block(aggregate, "defaults")
            .map(|defaults| parse_route_attributes(active_children(defaults).map(|child| child.text.as_str()))),
        routes: active_children(aggregate)
            .filter_map(|line| Some(parse_route(line.arg("route")?, line)))
            .collect(),
    }
}

fn parse_route(arg: &str, line: &ConfigLine) -> AggregateRoute {
    let (prefix, inline) = split_name(arg);
    let statements = route_statements(inline, line);
    let mut route = AggregateRoute {
        prefix: prefix.to_string(),
        policy: None,
        brief: false,
        full: false,
        discard: false,
        next_table: None,
        algorithm: None,
        attributes: RouteAttributes::default(),
    };
    for statement in &statements {
        let (keyword, arg) = split_keyword(statement);
        match keyword {
            "policy" => route.policy = Some(unquote(arg).to_string()),
            "brief" => route.brief = true,
            "full" => route.full = true,
            "discard" => route.discard = true,
            "next-table" => route.next_table = Some(unquote(arg).to_string()),
            "algorithm" => route.algorithm = Some(arg.to_string()),
            _ => {}
        }
    }
    route.attributes = parse_route_attributes(statements);
    route
}

#[cfg(test)]
mod tests {
    use crate::routing_options::routing_options;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
routing-options {
    aggregate {
        defaults {
            preference 130;
            community no-export;
        }
        route 10.0.0.0/8 {
            policy CONTRIBUTORS;
            brief;
            discard;
            metric 5;
            tag 9;
            passive;
            description "summary";
        }
        route 192.0.2.0/24 discard;
        route 198.51.100.0/24 next-table other.inet.0;
        route 2001:db8::/32 {
            algorithm 1;
            full;
        }
        inactive: route 203.0.113.0/24 discard;
    }
}
"#;

    fn aggregate() -> crate::routing_options::AggregateRoutes {
        routing_options(&juniper::parse_braces(CONFIG))
            .unwrap()
            .aggregate
            .unwrap()
    }

    #[test]
    fn extracts_defaults_and_block_route() {
        let aggregate = aggregate();
        let defaults = aggregate.defaults.unwrap();
        assert_eq!(defaults.preference.as_deref(), Some("130"));
        assert_eq!(defaults.community, ["no-export"]);

        assert_eq!(aggregate.routes.len(), 4);
        let route = &aggregate.routes[0];
        assert_eq!(route.prefix, "10.0.0.0/8");
        assert_eq!(route.policy.as_deref(), Some("CONTRIBUTORS"));
        assert!(route.brief);
        assert!(!route.full);
        assert!(route.discard);
        assert_eq!(route.attributes.metric.as_deref(), Some("5"));
        assert_eq!(route.attributes.tag.as_deref(), Some("9"));
        assert_eq!(route.attributes.active, Some(false));
        assert_eq!(route.attributes.description.as_deref(), Some("summary"));
    }

    #[test]
    fn extracts_one_line_routes_next_table_and_algorithm() {
        let aggregate = aggregate();
        assert!(aggregate.routes[1].discard);
        assert_eq!(aggregate.routes[2].next_table.as_deref(), Some("other.inet.0"));
        assert_eq!(aggregate.routes[3].algorithm.as_deref(), Some("1"));
        assert!(aggregate.routes[3].full);
    }

    #[test]
    fn no_aggregate_is_none() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    router-id 192.0.2.1;\n}\n",
        ))
        .unwrap();
        assert!(options.aggregate.is_none());
    }
}
