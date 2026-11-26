//! `static`
//! `routing-options { static { route <prefix> { ... } } }`.
//!
//! The module is named `static_routes` because `static` is a Rust keyword.
//!
//! # See also
//!
//! - [static (Routing Options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/static-edit-routing-options.html)

mod qualified_next_hop;
mod route;

pub use qualified_next_hop::QualifiedNextHop;
pub use route::StaticRoute;

use super::route_attributes::{parse_route_attributes, RouteAttributes};
use crate::{active_children, arg_of, child_block};
use asterust::config::ConfigLine;
use route::parse_route;

/// The `static` routes of a routing table.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct StaticRoutes {
    pub rib_group: Option<String>,
    /// The `defaults` that each route inherits unless it sets the attribute itself.
    pub defaults: Option<RouteAttributes>,
    pub routes: Vec<StaticRoute>,
}

pub(super) fn parse_static_routes(static_routes: &ConfigLine) -> StaticRoutes {
    StaticRoutes {
        rib_group: arg_of(static_routes, "rib-group").map(str::to_string),
        defaults: child_block(static_routes, "defaults")
            .map(|defaults| parse_route_attributes(active_children(defaults).map(|child| child.text.as_str()))),
        routes: active_children(static_routes)
            .filter_map(|line| Some(parse_route(line.arg("route")?, line)))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use crate::routing_options::routing_options;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
routing-options {
    static {
        rib-group INET-GROUP;
        defaults {
            no-readvertise;
            preference 10;
        }
        route 0.0.0.0/0 next-hop 192.0.2.1;
        route 10.0.0.0/8 discard;
        route 10.1.0.0/16 next-hop [ 192.0.2.2 192.0.2.3 ];
        route 10.2.0.0/16 {
            next-hop 192.0.2.4;
            preference 5 type 2;
            metric 3;
            tag 7;
            install;
            passive;
            community [ 65000:1 no-export ];
            description "to branch";
            qualified-next-hop 192.0.2.5 {
                preference 25;
                interface ge-0/0/1.0;
            }
            qualified-next-hop ge-0/0/2.0;
        }
        route 10.3.0.0/16 next-table other.inet.0;
        route 10.4.0.0/16 reject;
        route 10.5.0.0/16 receive;
        inactive: route 10.6.0.0/16 discard;
    }
}
"#;

    fn static_routes() -> crate::routing_options::StaticRoutes {
        routing_options(&juniper::parse_braces(CONFIG))
            .unwrap()
            .static_routes
            .unwrap()
    }

    #[test]
    fn extracts_rib_group_defaults_and_one_line_routes() {
        let statics = static_routes();
        assert_eq!(statics.rib_group.as_deref(), Some("INET-GROUP"));
        let defaults = statics.defaults.unwrap();
        assert_eq!(defaults.readvertise, Some(false));
        assert_eq!(defaults.preference.as_deref(), Some("10"));

        let prefixes: Vec<_> = statics.routes.iter().map(|r| r.prefix.as_str()).collect();
        assert_eq!(
            prefixes,
            [
                "0.0.0.0/0",
                "10.0.0.0/8",
                "10.1.0.0/16",
                "10.2.0.0/16",
                "10.3.0.0/16",
                "10.4.0.0/16",
                "10.5.0.0/16"
            ]
        );
        assert_eq!(statics.routes[0].next_hops, ["192.0.2.1"]);
        assert!(statics.routes[1].discard);
        assert_eq!(statics.routes[2].next_hops, ["192.0.2.2", "192.0.2.3"]);
    }

    #[test]
    fn extracts_block_route_attributes() {
        let route = &static_routes().routes[3];
        assert_eq!(route.next_hops, ["192.0.2.4"]);
        assert!(!route.discard);
        assert_eq!(route.attributes.preference.as_deref(), Some("5 type 2"));
        assert_eq!(route.attributes.metric.as_deref(), Some("3"));
        assert_eq!(route.attributes.tag.as_deref(), Some("7"));
        assert_eq!(route.attributes.install, Some(true));
        assert_eq!(route.attributes.active, Some(false));
        assert_eq!(route.attributes.community, ["65000:1", "no-export"]);
        assert_eq!(route.attributes.description.as_deref(), Some("to branch"));
        assert_eq!(route.attributes.readvertise, None);
    }

    #[test]
    fn extracts_qualified_next_hops() {
        let hops = &static_routes().routes[3].qualified_next_hops;
        assert_eq!(hops.len(), 2);
        assert_eq!(hops[0].next_hop, "192.0.2.5");
        assert_eq!(hops[0].preference.as_deref(), Some("25"));
        assert_eq!(hops[0].interface.as_deref(), Some("ge-0/0/1.0"));
        assert_eq!(hops[1].next_hop, "ge-0/0/2.0");
        assert_eq!(hops[1].preference, None);
    }

    #[test]
    fn extracts_next_table_reject_and_receive() {
        let statics = static_routes();
        assert_eq!(statics.routes[4].next_table.as_deref(), Some("other.inet.0"));
        assert!(statics.routes[5].reject);
        assert!(statics.routes[6].receive);
    }

    #[test]
    fn extracts_qualified_next_hop_written_on_the_route_line() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    static {\n        route 0.0.0.0/0 qualified-next-hop 192.0.2.9 {\n            preference 7;\n            order 2;\n        }\n    }\n}\n",
        ))
        .unwrap();
        let route = &options.static_routes.unwrap().routes[0];
        assert!(route.next_hops.is_empty());
        assert_eq!(route.qualified_next_hops[0].next_hop, "192.0.2.9");
        assert_eq!(route.qualified_next_hops[0].preference.as_deref(), Some("7"));
        assert_eq!(route.qualified_next_hops[0].order.as_deref(), Some("2"));
    }

    #[test]
    fn no_static_block_is_none() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    router-id 192.0.2.1;\n}\n",
        ))
        .unwrap();
        assert!(options.static_routes.is_none());
    }
}
