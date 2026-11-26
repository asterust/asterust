//! `rib`
//! `routing-options { rib <table> { ... } }`.
//!
//! # See also
//!
//! - [rib (General)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/rib-edit-routing-options.html)

use super::aggregate::{parse_aggregate_routes, AggregateRoutes};
use super::generate::{parse_generate_routes, GenerateRoutes};
use super::martians::{parse_martians, Martian};
use super::static_routes::{parse_static_routes, StaticRoutes};
use crate::{child_block, has_leaf, named_children};
use asterust::config::ConfigLine;

/// A `rib <table>`: the routes and martians that belong to one routing table.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Rib {
    /// The table name, e.g. `"inet6.0"` or `"VRF.inet.0"`.
    pub name: String,
    pub static_routes: Option<StaticRoutes>,
    pub aggregate: Option<AggregateRoutes>,
    pub generate: Option<GenerateRoutes>,
    pub martians: Vec<Martian>,
    pub policy_multipath: bool,
}

pub(super) fn parse_ribs(root: &ConfigLine) -> Vec<Rib> {
    named_children(root, "rib")
        .map(|(name, rib)| Rib {
            name: name.to_string(),
            static_routes: child_block(rib, "static").map(parse_static_routes),
            aggregate: child_block(rib, "aggregate").map(parse_aggregate_routes),
            generate: child_block(rib, "generate").map(parse_generate_routes),
            martians: parse_martians(rib),
            policy_multipath: has_leaf(rib, "policy-multipath"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::routing_options::routing_options;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_routes_martians_and_flags_per_table() {
        let options = routing_options(&juniper::parse_braces(
            r#"
routing-options {
    rib inet6.0 {
        static {
            route ::/0 next-hop 2001:db8::1;
        }
        aggregate {
            route 2001:db8::/32 discard;
        }
        martians {
            ::1/128 exact;
        }
        policy-multipath;
    }
    rib CUSTOMER.inet.0 {
        generate {
            route 0.0.0.0/0 policy DEFAULT-ROUTE;
        }
    }
}
"#,
        ))
        .unwrap();
        assert_eq!(options.ribs.len(), 2);

        let v6 = &options.ribs[0];
        assert_eq!(v6.name, "inet6.0");
        assert_eq!(v6.static_routes.as_ref().unwrap().routes[0].next_hops, ["2001:db8::1"]);
        assert_eq!(v6.aggregate.as_ref().unwrap().routes[0].prefix, "2001:db8::/32");
        assert_eq!(v6.martians[0].prefix, "::1/128");
        assert!(v6.policy_multipath);
        assert!(v6.generate.is_none());

        let customer = &options.ribs[1];
        assert_eq!(customer.name, "CUSTOMER.inet.0");
        assert_eq!(
            customer.generate.as_ref().unwrap().routes[0].policy.as_deref(),
            Some("DEFAULT-ROUTE")
        );
        assert!(customer.static_routes.is_none());
        assert!(!customer.policy_multipath);
    }

    #[test]
    fn no_ribs_is_empty() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    router-id 192.0.2.1;\n}\n",
        ))
        .unwrap();
        assert!(options.ribs.is_empty());
    }
}
