//! `mpls`
//! `protocols { mpls { ... } }`.
//!
//! This covers the router-wide options and the interfaces. LSPs, paths, traffic engineering database settings and
//! segment routing statements are not extracted yet.
//!
//! # See also
//!
//! - [mpls (Protocols)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/mpls-edit-protocols.html)
//! - [admin-groups](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/admin-groups-edit-protocols-mpls.html)
//! - [explicit-null (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/explicit-null-edit-protocols-mpls.html)
//! - [no-cspf](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/no-cspf-edit-protocols-mpls.html)
//! - [no-propagate-ttl](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/no-propagate-ttl-edit-protocols-mpls.html)
//! - [no-decrement-ttl](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/no-decrement-ttl-edit-protocols-mpls.html)
//! - [ipv6-tunneling](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/ipv6-tunneling-edit-protocols-mpls.html)
//! - [traffic-engineering (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/traffic-engineering-edit-protocols-mpls.html)
//! - [optimize-timer (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/optimize-timer-edit-protocols-mpls.html)
//! - [optimize-switchover-delay](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/optimize-switchover-delay-edit-protocols-mpls.html)
//! - [optimize-hold-dead-delay](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/optimize-hold-dead-delay-edit-protocols-mpls.html)
//! - [smart-optimize-timer](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/smart-optimize-timer-edit-protocols-mpls.html)
//! - [rsvp-error-hold-time](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/rsvp-error-hold-time-edit-protocols-mpls.html)
//! - [expand-loose-hop](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/expand-loose-hop-edit-protocols-mpls.html)
//! - [disable (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/disable-edit-protocols-mpls.html)
//! - [Basic MPLS Configuration](https://networkingdocs.hpe.com/documentation/us/en/software/junos/mpls/topics/topic-map/mpls-basic-configuration.html)

mod interface;
mod log_updown;
mod path_mtu;

pub use interface::MplsInterface;
pub use log_updown::MplsLogUpdown;
pub use path_mtu::MplsPathMtu;

use crate::{arg_of, child_block, has_leaf, named_children, split_name, top_level};
use asterust::config::{Config, ConfigLine};
use interface::parse_interface;
use log_updown::parse_log_updown;
use path_mtu::parse_path_mtu;

/// An `admin-groups` entry: a link color name and its bit.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct MplsAdminGroup {
    pub name: String,
    /// 0 through 31.
    pub value: String,
}

/// The `protocols mpls` configuration. The `label-switched-path`, `path`, `static-label-switched-path`,
/// `container-label-switched-path`, `traffic-engineering` database, `diffserv-te`, `statistics`, `traceoptions`,
/// `class-of-service`, `ldp-tunneling` and segment routing statements are not extracted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Mpls {
    pub disable: bool,
    pub interfaces: Vec<MplsInterface>,
    pub admin_groups: Vec<MplsAdminGroup>,
    /// Advertise label 0 instead of implicit null to the egress router.
    pub explicit_null: bool,
    /// Disable constrained-path LSP computation.
    pub no_cspf: bool,
    pub no_propagate_ttl: bool,
    pub no_decrement_ttl: bool,
    pub ipv6_tunneling: bool,
    pub expand_loose_hop: bool,
    /// Which destinations MPLS engineers traffic for: `bgp`, `bgp-igp`, `bgp-igp-both-ribs` or `mpls-forwarding`.
    pub traffic_engineering: Option<String>,
    /// Seconds.
    pub optimize_timer: Option<String>,
    /// Seconds.
    pub optimize_switchover_delay: Option<String>,
    /// Seconds.
    pub optimize_hold_dead_delay: Option<String>,
    /// Seconds.
    pub smart_optimize_timer: Option<String>,
    /// Seconds.
    pub rsvp_error_hold_time: Option<String>,
    pub path_mtu: Option<MplsPathMtu>,
    pub log_updown: Option<MplsLogUpdown>,
}

/// Returns the `protocols mpls` block, or `None` if MPLS is not configured.
///
/// # Examples
///
/// ```
/// use asterust::config::juniper;
/// use asterust_junos::protocols::mpls::mpls;
///
/// let config = juniper::parse_braces("protocols {\n    mpls {\n        explicit-null;\n        interface ge-0/0/5.0;\n    }\n}\n");
/// let mpls = mpls(&config).unwrap();
/// assert!(mpls.explicit_null);
/// assert_eq!(mpls.interfaces[0].name, "ge-0/0/5.0");
/// ```
pub fn mpls(config: &Config) -> Option<Mpls> {
    let mpls = child_block(top_level(config, "protocols")?, "mpls")?;
    let field = |keyword: &str| arg_of(mpls, keyword).map(str::to_string);
    Some(Mpls {
        disable: has_leaf(mpls, "disable"),
        interfaces: named_children(mpls, "interface")
            .map(|(name, interface)| parse_interface(name, interface))
            .collect(),
        admin_groups: parse_admin_groups(mpls),
        explicit_null: has_leaf(mpls, "explicit-null"),
        no_cspf: has_leaf(mpls, "no-cspf"),
        no_propagate_ttl: has_leaf(mpls, "no-propagate-ttl"),
        no_decrement_ttl: has_leaf(mpls, "no-decrement-ttl"),
        ipv6_tunneling: has_leaf(mpls, "ipv6-tunneling"),
        expand_loose_hop: has_leaf(mpls, "expand-loose-hop"),
        traffic_engineering: field("traffic-engineering"),
        optimize_timer: field("optimize-timer"),
        optimize_switchover_delay: field("optimize-switchover-delay"),
        optimize_hold_dead_delay: field("optimize-hold-dead-delay"),
        smart_optimize_timer: field("smart-optimize-timer"),
        rsvp_error_hold_time: field("rsvp-error-hold-time"),
        path_mtu: parse_path_mtu(mpls),
        log_updown: parse_log_updown(mpls),
    })
}

fn parse_admin_groups(mpls: &ConfigLine) -> Vec<MplsAdminGroup> {
    let Some(block) = child_block(mpls, "admin-groups") else {
        return Vec::new();
    };
    crate::active_children(block)
        .map(|group| {
            let (name, value) = split_name(&group.text);
            MplsAdminGroup {
                name: name.to_string(),
                value: value.to_string(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::juniper;

    // Provider switch `show configuration` (MPLS part) of "Example: Configuring MPLS on EX8200 and EX4500 Switches"
    // (MPLS Applications User Guide, Basic MPLS Configuration), with the local PE's label-switched-path added from its
    // own results; see the README "Sources".
    const DOC_BASIC: &str = r#"
protocols {
    rsvp {
        interface lo0.0;
        interface ge-0/0/5.0;
    }
    mpls {
        label-switched-path lsp_to_pe2_ge1 {
            to 127.1.1.3;
        }
        interface ge-0/0/5.0;
        interface ge-0/0/6.0;
        interface ae0.0;
    }
    ospf {
        traffic-engineering;
    }
}
"#;

    // Not from documentation: written from the CLI Reference pages of the statements. See the README "Sources".
    const EXTRA: &str = r#"
protocols {
    mpls {
        explicit-null;
        no-cspf;
        no-propagate-ttl;
        no-decrement-ttl;
        ipv6-tunneling;
        expand-loose-hop;
        traffic-engineering bgp-igp;
        optimize-timer 3600;
        optimize-switchover-delay 5;
        optimize-hold-dead-delay 10;
        smart-optimize-timer 180;
        rsvp-error-hold-time 25;
        admin-groups {
            gold 1;
            silver 2;
        }
        path-mtu {
            allow-fragmentation;
            rsvp {
                mtu-signaling;
            }
        }
        log-updown {
            no-trap {
                mpls-lsp-traps;
            }
            no-syslog;
            trap;
            trap-path-down;
        }
        interface all {
            admin-group [ gold silver ];
            srlg SRLG-A;
        }
        interface fxp0.0 {
            disable;
        }
    }
}
"#;

    #[test]
    fn reads_the_guide_interfaces() {
        let mpls = mpls(&juniper::parse_braces(DOC_BASIC)).unwrap();
        assert!(!mpls.disable);
        let names: Vec<&str> = mpls.interfaces.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, ["ge-0/0/5.0", "ge-0/0/6.0", "ae0.0"]);
        assert!(mpls.interfaces.iter().all(|i| !i.disable));
        assert!(!mpls.explicit_null);
        assert_eq!(mpls.path_mtu, None);
    }

    #[test]
    fn reads_the_flags_and_timers() {
        let mpls = mpls(&juniper::parse_braces(EXTRA)).unwrap();
        assert!(mpls.explicit_null);
        assert!(mpls.no_cspf);
        assert!(mpls.no_propagate_ttl);
        assert!(mpls.no_decrement_ttl);
        assert!(mpls.ipv6_tunneling);
        assert!(mpls.expand_loose_hop);
        assert_eq!(mpls.traffic_engineering.as_deref(), Some("bgp-igp"));
        assert_eq!(mpls.optimize_timer.as_deref(), Some("3600"));
        assert_eq!(mpls.optimize_switchover_delay.as_deref(), Some("5"));
        assert_eq!(mpls.optimize_hold_dead_delay.as_deref(), Some("10"));
        assert_eq!(mpls.smart_optimize_timer.as_deref(), Some("180"));
        assert_eq!(mpls.rsvp_error_hold_time.as_deref(), Some("25"));
    }

    #[test]
    fn reads_admin_groups_and_interface_options() {
        let mpls = mpls(&juniper::parse_braces(EXTRA)).unwrap();
        assert_eq!(mpls.admin_groups.len(), 2);
        assert_eq!(mpls.admin_groups[0].name, "gold");
        assert_eq!(mpls.admin_groups[0].value, "1");

        let all = &mpls.interfaces[0];
        assert_eq!(all.name, "all");
        assert_eq!(all.admin_groups, ["gold", "silver"]);
        assert_eq!(all.srlg, ["SRLG-A"]);
        assert!(mpls.interfaces[1].disable);
    }

    #[test]
    fn reads_path_mtu_and_log_updown() {
        let mpls = mpls(&juniper::parse_braces(EXTRA)).unwrap();
        let mtu = mpls.path_mtu.unwrap();
        assert!(mtu.allow_fragmentation);
        assert!(mtu.rsvp_mtu_signaling);

        let log = mpls.log_updown.unwrap();
        assert_eq!(log.syslog, Some(false));
        assert!(log.trap);
        assert!(log.trap_path_down);
        assert!(!log.trap_path_up);
        assert!(log.no_trap_mpls_lsp_traps);
        assert!(!log.no_trap_rfc3812_traps);
    }

    #[test]
    fn reads_a_bare_mpls_block() {
        let config = juniper::parse_braces("protocols {\n    mpls;\n}\n");
        assert_eq!(mpls(&config), Some(Mpls::default()));
    }

    #[test]
    fn no_mpls_is_none() {
        let config = juniper::parse_braces("protocols {\n    ospf {\n        area 0.0.0.0;\n    }\n}\n");
        assert_eq!(mpls(&config), None);
    }
}
