//! `forwarding-table`
//! `routing-options { forwarding-table { ... } }`.
//!
//! # See also
//!
//! - [forwarding-table](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/forwarding-table-edit-routing-options.html)

use crate::{arg_of, args_of, has_leaf, list_values, toggle};
use asterust::config::ConfigLine;

/// The global `forwarding-table` settings. `chained-composite-next-hop` and the EVPN statements are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ForwardingTable {
    /// Policies that decide which routes are installed in the forwarding table.
    pub export: Vec<String>,
    pub ecmp_fast_reroute: bool,
    /// `Some(true)` for `indirect-next-hop`, `Some(false)` for `no-indirect-next-hop`.
    pub indirect_next_hop: Option<bool>,
    pub indirect_next_hop_change_acknowledgements: Option<bool>,
    pub krt_nexthop_ack_timeout: Option<String>,
    /// Seconds.
    pub remnant_holdtime: Option<String>,
    /// Seconds.
    pub multicast_composite_delete_defer_timeout: Option<String>,
    /// `active-paths` or `feasible-paths`.
    pub unicast_reverse_path: Option<String>,
    /// Merges SRv6 chain next hops in the platform layer.
    pub srv6_chain_merge: bool,
}

pub(super) fn parse_forwarding_table(table: &ConfigLine) -> ForwardingTable {
    let field = |keyword: &str| arg_of(table, keyword).map(str::to_string);
    ForwardingTable {
        export: args_of(table, "export").into_iter().flat_map(list_values).collect(),
        ecmp_fast_reroute: has_leaf(table, "ecmp-fast-reroute"),
        indirect_next_hop: toggle(table, "indirect-next-hop"),
        indirect_next_hop_change_acknowledgements: toggle(table, "indirect-next-hop-change-acknowledgements"),
        krt_nexthop_ack_timeout: field("krt-nexthop-ack-timeout"),
        remnant_holdtime: field("remnant-holdtime"),
        multicast_composite_delete_defer_timeout: field("multicast-composite-delete-defer-timeout"),
        unicast_reverse_path: field("unicast-reverse-path"),
        srv6_chain_merge: has_leaf(table, "srv6-chain-merge"),
    }
}

#[cfg(test)]
mod tests {
    use crate::routing_options::routing_options;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_forwarding_table_settings() {
        let options = routing_options(&juniper::parse_braces(
            r#"
routing-options {
    forwarding-table {
        export [ LOAD-BALANCE ECMP ];
        ecmp-fast-reroute;
        no-indirect-next-hop;
        indirect-next-hop-change-acknowledgements;
        krt-nexthop-ack-timeout 20;
        remnant-holdtime 300;
        multicast-composite-delete-defer-timeout 60;
        unicast-reverse-path feasible-paths;
        srv6-chain-merge;
    }
}
"#,
        ))
        .unwrap();
        let table = options.forwarding_table.unwrap();
        assert_eq!(table.export, ["LOAD-BALANCE", "ECMP"]);
        assert!(table.ecmp_fast_reroute);
        assert_eq!(table.indirect_next_hop, Some(false));
        assert_eq!(table.indirect_next_hop_change_acknowledgements, Some(true));
        assert_eq!(table.krt_nexthop_ack_timeout.as_deref(), Some("20"));
        assert_eq!(table.remnant_holdtime.as_deref(), Some("300"));
        assert_eq!(table.multicast_composite_delete_defer_timeout.as_deref(), Some("60"));
        assert_eq!(table.unicast_reverse_path.as_deref(), Some("feasible-paths"));
        assert!(table.srv6_chain_merge);
    }

    #[test]
    fn single_export_policy_needs_no_brackets() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    forwarding-table {\n        export LOAD-BALANCE;\n    }\n}\n",
        ))
        .unwrap();
        let table = options.forwarding_table.unwrap();
        assert_eq!(table.export, ["LOAD-BALANCE"]);
        assert_eq!(table.indirect_next_hop, None);
        assert!(!table.ecmp_fast_reroute);
    }

    #[test]
    fn no_forwarding_table_is_none() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    router-id 192.0.2.1;\n}\n",
        ))
        .unwrap();
        assert!(options.forwarding_table.is_none());
    }
}
