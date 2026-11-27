//! `interface`
//! `protocols { isis { interface <name | all> { ... } } }`.
//!
//! # See also
//!
//! - [interface (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/interface-edit-protocols-isis.html)
//! - [passive (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/passive-edit-protocols-isis.html)
//! - [link-protection (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/link-protection-edit-protocols-isis.html)
//! - [strict-dual-isis (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/strict-dual-isis-edit-protocols-isis.html)
//! - [delay-metric](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/delay-metric-protocols-isis-interface.html)
//! - [no-advertise-adjacency-segment (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/no-advertise-adjacency-segment-edit-protocols-isis-interface-level.html)
//! - [point-to-point](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/point-to-point-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [lsp-interval](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/lsp-interval-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [csnp-interval](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/csnp-interval-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [hello-padding](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/hello-padding-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [checksum (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/checksum-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [mesh-group (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/mesh-group-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [ldp-synchronization](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/ldp-synchronization-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [no-adjacency-holddown](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/no-adjacency-holddown-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [node-link-protection (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/node-link-protection-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [no-eligible-backup (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/no-eligible-backup-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [no-ipv4-multicast](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/no-ipv4-multicast-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [no-ipv6-multicast](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/no-ipv6-multicast-edit-protocols-isis.html), from the Junos 12.3 archive

mod bfd;
mod level;

pub use bfd::IsisBfd;
pub use level::IsisInterfaceLevel;

use crate::{arg_of, args_of, has_leaf, list_values, named_children, statement_words, value_after};
use asterust::config::ConfigLine;
use bfd::parse_bfd;
use level::parse_level;

/// An `interface <name>` entry under IS-IS. Values are kept as written. The `delay-measurement` block and the
/// `application-specific` blocks of a level are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct IsisInterface {
    pub name: String,
    pub passive: bool,
    pub point_to_point: bool,
    pub disable: bool,
    /// From `passive { remote-node-id ...; }`.
    pub passive_remote_node_id: Option<String>,
    /// From `passive { remote-node-iso ...; }`.
    pub passive_remote_node_iso: Option<String>,
    /// Milliseconds.
    pub lsp_interval: Option<String>,
    /// Seconds, or `disable`.
    pub csnp_interval: Option<String>,
    /// `adaptive`, `disable`, `loose` or `strict`.
    pub hello_padding: Option<String>,
    pub max_hello_size: Option<String>,
    /// A number, or `blocked`.
    pub mesh_group: Option<String>,
    pub interface_group_holddown_delay: Option<String>,
    pub srlg: Vec<String>,
    pub delay_metric: Option<String>,
    pub strict_dual_isis: bool,
    /// Seconds, from `strict-dual-isis holddown ...`.
    pub strict_dual_isis_holddown: Option<String>,
    pub ldp_synchronization_disable: bool,
    /// Seconds.
    pub ldp_synchronization_hold_time: Option<String>,
    pub bfd_liveness_detection: Option<IsisBfd>,
    /// The `level <1|2>` entries of the interface.
    pub levels: Vec<IsisInterfaceLevel>,
    pub checksum: bool,
    pub flood_group: bool,
    pub link_protection: bool,
    pub node_link_protection: bool,
    pub no_adjacency_holddown: bool,
    pub no_eligible_backup: bool,
    pub no_eligible_remote_backup: bool,
    pub no_ipv4_multicast: bool,
    pub no_ipv6_multicast: bool,
    pub no_ipv6_unicast: bool,
    pub no_unicast_topology: bool,
    pub no_advertise_adjacency_segment: bool,
}

pub(super) fn parse_interface(name: &str, iface: &ConfigLine) -> IsisInterface {
    let field = |keyword: &str| arg_of(iface, keyword).map(str::to_string);
    let present = |keyword: &str| statement_words(iface, keyword).is_some();
    let passive = statement_words(iface, "passive");
    let ldp = statement_words(iface, "ldp-synchronization");
    let strict = statement_words(iface, "strict-dual-isis");
    IsisInterface {
        name: name.to_string(),
        passive: has_leaf(iface, "passive"),
        point_to_point: has_leaf(iface, "point-to-point"),
        disable: has_leaf(iface, "disable"),
        passive_remote_node_id: passive.as_ref().and_then(|words| value_after(words, "remote-node-id")),
        passive_remote_node_iso: passive.as_ref().and_then(|words| value_after(words, "remote-node-iso")),
        lsp_interval: field("lsp-interval"),
        csnp_interval: field("csnp-interval"),
        hello_padding: field("hello-padding"),
        max_hello_size: field("max-hello-size"),
        mesh_group: field("mesh-group"),
        interface_group_holddown_delay: field("interface-group-holddown-delay"),
        srlg: args_of(iface, "srlg").into_iter().flat_map(list_values).collect(),
        delay_metric: field("delay-metric"),
        strict_dual_isis: strict.is_some(),
        strict_dual_isis_holddown: strict.and_then(|words| value_after(&words, "holddown")),
        ldp_synchronization_disable: ldp.as_ref().is_some_and(|words| words.contains(&"disable")),
        ldp_synchronization_hold_time: ldp.and_then(|words| value_after(&words, "hold-time")),
        bfd_liveness_detection: parse_bfd(iface),
        levels: named_children(iface, "level")
            .map(|(arg, level)| parse_level(arg, level))
            .collect(),
        checksum: has_leaf(iface, "checksum"),
        flood_group: has_leaf(iface, "flood-group"),
        link_protection: present("link-protection"),
        node_link_protection: present("node-link-protection"),
        no_adjacency_holddown: has_leaf(iface, "no-adjacency-holddown"),
        no_eligible_backup: has_leaf(iface, "no-eligible-backup"),
        no_eligible_remote_backup: has_leaf(iface, "no-eligible-remote-backup"),
        no_ipv4_multicast: has_leaf(iface, "no-ipv4-multicast"),
        no_ipv6_multicast: has_leaf(iface, "no-ipv6-multicast"),
        no_ipv6_unicast: has_leaf(iface, "no-ipv6-unicast"),
        no_unicast_topology: has_leaf(iface, "no-unicast-topology"),
        no_advertise_adjacency_segment: has_leaf(iface, "no-advertise-adjacency-segment"),
    }
}

#[cfg(test)]
mod tests {
    use crate::protocols::isis::isis;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    isis {
        interface ge-0/0/0.0 {
            point-to-point;
            lsp-interval 50;
            csnp-interval 5;
            hello-padding adaptive;
            max-hello-size 1400;
            mesh-group blocked;
            srlg [ SRLG-A ];
            srlg SRLG-B;
            delay-metric 1000;
            strict-dual-isis holddown 30;
            checksum;
            link-protection;
            no-eligible-backup;
            no-ipv6-unicast;
            ldp-synchronization {
                hold-time 60;
            }
        }
        interface lo0.0 {
            passive {
                remote-node-id 192.0.2.9;
                remote-node-iso 49.0001.0000.0000.0009;
            }
        }
        interface ge-0/0/1.0 {
            disable;
            csnp-interval disable;
            ldp-synchronization {
                disable;
            }
        }
        interface all;
    }
}
"#;

    #[test]
    fn reads_the_timers_and_flags() {
        let isis = isis(&juniper::parse_braces(CONFIG)).unwrap();
        let iface = &isis.interfaces[0];
        assert!(iface.point_to_point);
        assert_eq!(iface.lsp_interval.as_deref(), Some("50"));
        assert_eq!(iface.csnp_interval.as_deref(), Some("5"));
        assert_eq!(iface.hello_padding.as_deref(), Some("adaptive"));
        assert_eq!(iface.max_hello_size.as_deref(), Some("1400"));
        assert_eq!(iface.mesh_group.as_deref(), Some("blocked"));
        assert_eq!(iface.delay_metric.as_deref(), Some("1000"));
        assert!(iface.strict_dual_isis);
        assert_eq!(iface.strict_dual_isis_holddown.as_deref(), Some("30"));
        assert!(iface.checksum);
        assert!(iface.link_protection);
        assert!(iface.no_eligible_backup);
        assert!(iface.no_ipv6_unicast);
        assert!(!iface.node_link_protection);
        assert_eq!(iface.ldp_synchronization_hold_time.as_deref(), Some("60"));
        assert!(!iface.ldp_synchronization_disable);
    }

    #[test]
    fn reads_the_passive_block() {
        let isis = isis(&juniper::parse_braces(CONFIG)).unwrap();
        let iface = &isis.interfaces[1];
        assert!(iface.passive);
        assert_eq!(iface.passive_remote_node_id.as_deref(), Some("192.0.2.9"));
        assert_eq!(iface.passive_remote_node_iso.as_deref(), Some("49.0001.0000.0000.0009"));
    }

    #[test]
    fn reads_disabled_interfaces_and_all() {
        let isis = isis(&juniper::parse_braces(CONFIG)).unwrap();
        let disabled = &isis.interfaces[2];
        assert!(disabled.disable);
        assert_eq!(disabled.csnp_interval.as_deref(), Some("disable"));
        assert!(disabled.ldp_synchronization_disable);
        let all = &isis.interfaces[3];
        assert_eq!(all.name, "all");
        assert!(!all.strict_dual_isis);
        assert_eq!(all.bfd_liveness_detection, None);
        assert!(all.levels.is_empty());
    }

    #[test]
    fn reads_the_srlg_entries() {
        let isis = isis(&juniper::parse_braces(CONFIG)).unwrap();
        assert_eq!(isis.interfaces[0].srlg, ["SRLG-A", "SRLG-B"]);
        assert!(isis.interfaces[1].srlg.is_empty());
    }
}
