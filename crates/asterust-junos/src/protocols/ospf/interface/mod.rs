//! `interface`
//! `protocols { ospf { area <id> { interface <name> { ... } } } }`.
//!
//! # See also
//!
//! - [interface (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/interface-edit-protocols-ospf.html)
//! - [interface-type (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/interface-type-edit-protocols-ospf.html)
//! - [passive (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/passive-edit-protocols-ospf.html)
//! - [traffic-engineering (Passive TE Mode)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/traffic-engineering-edit-protocols-ospf-passive.html)
//! - [topology (OSPF Interface)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/topology-edit-protocols-ospf-interface.html)
//! - [link-protection (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/link-protection-edit-protocols-ospf.html)
//! - [no-advertise-adjacency-segment (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/no-advertise-adjacency-segment-edit-protocols-ospf-interface-level.html)

mod authentication;
mod bfd;

pub use authentication::{OspfAuthentication, OspfMd5Key};
pub use bfd::OspfBfd;

use crate::{arg_of, has_leaf, named_children, split_name, statement_words, value_after};
use asterust::config::ConfigLine;
use authentication::parse_authentication;
use bfd::parse_bfd;

/// An `interface <name>` entry under an OSPF area. Values are kept as written. The `passive` traffic-engineering
/// settings, `bandwidth-based-metrics`, `lan-neighbor` and `ipv4-adjacency-segment` are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfInterface {
    pub name: String,
    pub passive: bool,
    pub metric: Option<u32>,
    /// OSPF is disabled on this interface.
    pub disable: bool,
    pub priority: Option<String>,
    /// Seconds.
    pub hello_interval: Option<String>,
    /// Seconds.
    pub dead_interval: Option<String>,
    /// Seconds.
    pub retransmit_interval: Option<String>,
    /// Seconds.
    pub transit_delay: Option<String>,
    /// Seconds, for NBMA interfaces.
    pub poll_interval: Option<String>,
    pub te_metric: Option<String>,
    pub mtu: Option<String>,
    /// `nbma`, `p2mp`, `p2mp-over-lan` or `p2p`.
    pub interface_type: Option<String>,
    pub ipsec_sa: Option<String>,
    pub authentication: Option<OspfAuthentication>,
    pub bfd_liveness_detection: Option<OspfBfd>,
    /// `ldp-synchronization { disable; }`.
    pub ldp_synchronization_disable: bool,
    /// Seconds, from `ldp-synchronization { hold-time ...; }`.
    pub ldp_synchronization_hold_time: Option<String>,
    pub neighbors: Vec<OspfNeighbor>,
    pub topologies: Vec<OspfInterfaceTopology>,
    pub link_protection: bool,
    pub node_link_protection: bool,
    pub no_eligible_backup: bool,
    pub no_eligible_remote_backup: bool,
    pub secondary: bool,
    pub flood_reduction: bool,
    pub demand_circuit: bool,
    pub no_interface_state_traps: bool,
    pub no_neighbor_down_notification: bool,
    pub no_advertise_adjacency_segment: bool,
    pub dynamic_neighbors: bool,
    pub strict_bfd: bool,
}

/// A `neighbor <address> [eligible]` of a non-broadcast interface.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfNeighbor {
    pub address: String,
    pub eligible: bool,
}

/// A `topology <name> { metric ...; }` of an interface, e.g. `ipv4-multicast`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfInterfaceTopology {
    pub name: String,
    pub metric: Option<String>,
}

pub(super) fn parse_interface(name: &str, iface: &ConfigLine) -> OspfInterface {
    let field = |keyword: &str| arg_of(iface, keyword).map(str::to_string);
    let present = |keyword: &str| statement_words(iface, keyword).is_some();
    let ldp = statement_words(iface, "ldp-synchronization");
    OspfInterface {
        name: name.to_string(),
        passive: has_leaf(iface, "passive"),
        metric: arg_of(iface, "metric").and_then(|metric| metric.parse().ok()),
        disable: has_leaf(iface, "disable"),
        priority: field("priority"),
        hello_interval: field("hello-interval"),
        dead_interval: field("dead-interval"),
        retransmit_interval: field("retransmit-interval"),
        transit_delay: field("transit-delay"),
        poll_interval: field("poll-interval"),
        te_metric: field("te-metric"),
        mtu: field("mtu"),
        interface_type: field("interface-type"),
        ipsec_sa: field("ipsec-sa"),
        authentication: parse_authentication(iface),
        bfd_liveness_detection: parse_bfd(iface),
        ldp_synchronization_disable: ldp.as_ref().is_some_and(|words| words.contains(&"disable")),
        ldp_synchronization_hold_time: ldp.and_then(|words| value_after(&words, "hold-time")),
        neighbors: named_children(iface, "neighbor")
            .map(|(arg, neighbor)| {
                let (address, rest) = split_name(arg);
                OspfNeighbor {
                    address: address.to_string(),
                    eligible: rest.split_whitespace().any(|word| word == "eligible") || has_leaf(neighbor, "eligible"),
                }
            })
            .collect(),
        topologies: named_children(iface, "topology")
            .map(|(name, topology)| OspfInterfaceTopology {
                name: name.to_string(),
                metric: arg_of(topology, "metric").map(str::to_string),
            })
            .collect(),
        link_protection: present("link-protection"),
        node_link_protection: present("node-link-protection"),
        no_eligible_backup: has_leaf(iface, "no-eligible-backup"),
        no_eligible_remote_backup: has_leaf(iface, "no-eligible-remote-backup"),
        secondary: has_leaf(iface, "secondary"),
        flood_reduction: has_leaf(iface, "flood-reduction"),
        demand_circuit: has_leaf(iface, "demand-circuit"),
        no_interface_state_traps: has_leaf(iface, "no-interface-state-traps"),
        no_neighbor_down_notification: has_leaf(iface, "no-neighbor-down-notification"),
        no_advertise_adjacency_segment: has_leaf(iface, "no-advertise-adjacency-segment"),
        dynamic_neighbors: has_leaf(iface, "dynamic-neighbors"),
        strict_bfd: has_leaf(iface, "strict-bfd"),
    }
}

#[cfg(test)]
mod tests {
    use crate::protocols::ospf::ospf;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    ospf {
        area 0.0.0.0 {
            interface ge-0/0/0.0 {
                interface-type p2p;
                metric 10;
                priority 200;
                hello-interval 5;
                dead-interval 20;
                retransmit-interval 3;
                transit-delay 2;
                te-metric 15;
                mtu 1400;
                ipsec-sa SA-OSPF;
                link-protection;
                no-eligible-backup;
                strict-bfd;
                ldp-synchronization {
                    hold-time 30;
                }
            }
            interface ge-0/0/1.0 {
                interface-type nbma;
                poll-interval 60;
                neighbor 10.0.0.2 eligible;
                neighbor 10.0.0.3;
                secondary;
                flood-reduction;
                demand-circuit;
                no-interface-state-traps;
                topology ipv4-multicast {
                    metric 25;
                }
                ldp-synchronization {
                    disable;
                }
            }
            interface lo0.0 {
                passive {
                    traffic-engineering {
                        remote-node-id 10.0.0.9;
                    }
                }
            }
            interface ge-0/0/2.0 {
                disable;
            }
            interface ge-0/0/3.0;
        }
    }
}
"#;

    #[test]
    fn reads_the_timers_and_basic_options() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let iface = &ospf.areas[0].interfaces[0];
        assert_eq!(iface.interface_type.as_deref(), Some("p2p"));
        assert_eq!(iface.metric, Some(10));
        assert_eq!(iface.priority.as_deref(), Some("200"));
        assert_eq!(iface.hello_interval.as_deref(), Some("5"));
        assert_eq!(iface.dead_interval.as_deref(), Some("20"));
        assert_eq!(iface.retransmit_interval.as_deref(), Some("3"));
        assert_eq!(iface.transit_delay.as_deref(), Some("2"));
        assert_eq!(iface.te_metric.as_deref(), Some("15"));
        assert_eq!(iface.mtu.as_deref(), Some("1400"));
        assert_eq!(iface.ipsec_sa.as_deref(), Some("SA-OSPF"));
        assert!(iface.link_protection);
        assert!(iface.no_eligible_backup);
        assert!(iface.strict_bfd);
        assert!(!iface.node_link_protection);
        assert!(!iface.ldp_synchronization_disable);
        assert_eq!(iface.ldp_synchronization_hold_time.as_deref(), Some("30"));
    }

    #[test]
    fn reads_the_non_broadcast_options() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let iface = &ospf.areas[0].interfaces[1];
        assert_eq!(iface.interface_type.as_deref(), Some("nbma"));
        assert_eq!(iface.poll_interval.as_deref(), Some("60"));
        assert_eq!(iface.neighbors.len(), 2);
        assert_eq!(iface.neighbors[0].address, "10.0.0.2");
        assert!(iface.neighbors[0].eligible);
        assert!(!iface.neighbors[1].eligible);
        assert!(iface.secondary);
        assert!(iface.flood_reduction);
        assert!(iface.demand_circuit);
        assert!(iface.no_interface_state_traps);
        assert!(iface.ldp_synchronization_disable);
        assert_eq!(iface.topologies[0].name, "ipv4-multicast");
        assert_eq!(iface.topologies[0].metric.as_deref(), Some("25"));
    }

    #[test]
    fn reads_passive_blocks_and_disabled_interfaces() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let interfaces = &ospf.areas[0].interfaces;
        assert!(interfaces[2].passive);
        assert!(interfaces[3].disable);
        assert!(!interfaces[3].passive);
        let bare = &interfaces[4];
        assert_eq!(bare.priority, None);
        assert_eq!(bare.authentication, None);
        assert_eq!(bare.bfd_liveness_detection, None);
        assert!(bare.neighbors.is_empty());
    }
}
