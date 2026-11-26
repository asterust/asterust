//! `... address <address> { vrrp-group <id> { ... } }` and `vrrp-inet6-group <id> { ... }`.

mod track;

pub use track::{BandwidthThreshold, TrackedInterface, TrackedRoute, VrrpTrack};

use crate::{arg_of, args_of, child_block, has_leaf, list_values, named_children, toggle};
use asterust::config::ConfigLine;
use track::parse_track;

/// The `vrrp-inherit-from` block of a VRRP group.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct VrrpInheritFrom {
    pub active_group: Option<String>,
    pub active_interface: Option<String>,
}

/// A `vrrp-group` (under `family inet`) or `vrrp-inet6-group` (under `family inet6`). The `authentication-key` is not
/// extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct VrrpGroup {
    pub group: String,
    /// Written as `vrrp-inet6-group`.
    pub inet6: bool,
    /// `Some(true)` for `accept-data`, `Some(false)` for `no-accept-data`.
    pub accept_data: Option<bool>,
    /// Seconds, `vrrp-group` only.
    pub advertise_interval: Option<String>,
    /// Milliseconds, `vrrp-inet6-group` only.
    pub inet6_advertise_interval: Option<String>,
    pub advertisements_threshold: Option<String>,
    /// Milliseconds.
    pub fast_interval: Option<String>,
    pub priority: Option<String>,
    pub preferred: bool,
    /// `md5` or `simple`.
    pub authentication_type: Option<String>,
    /// An `authentication-key` is configured.
    pub has_authentication_key: bool,
    /// `Some(true)` for `preempt`, `Some(false)` for `no-preempt`.
    pub preempt: Option<bool>,
    /// Seconds, from `preempt { hold-time ...; }`.
    pub preempt_hold_time: Option<String>,
    /// From `virtual-address` or `virtual-inet6-address`.
    pub virtual_addresses: Vec<String>,
    pub virtual_link_local_address: Option<String>,
    pub track: Option<VrrpTrack>,
    pub inherit_from: Option<VrrpInheritFrom>,
}

/// Every `vrrp-group` and `vrrp-inet6-group` under `address`.
pub(super) fn parse_vrrp_groups(address: &ConfigLine) -> Vec<VrrpGroup> {
    let groups = |keyword: &'static str, inet6: bool| {
        named_children(address, keyword).map(move |(id, group)| parse_group(id, group, inet6))
    };
    groups("vrrp-group", false)
        .chain(groups("vrrp-inet6-group", true))
        .collect()
}

fn parse_group(id: &str, group: &ConfigLine, inet6: bool) -> VrrpGroup {
    let field = |keyword: &str| arg_of(group, keyword).map(str::to_string);
    VrrpGroup {
        group: id.to_string(),
        inet6,
        accept_data: toggle(group, "accept-data"),
        advertise_interval: field("advertise-interval"),
        inet6_advertise_interval: field("inet6-advertise-interval"),
        advertisements_threshold: field("advertisements-threshold"),
        fast_interval: field("fast-interval"),
        priority: field("priority"),
        preferred: has_leaf(group, "preferred"),
        authentication_type: field("authentication-type"),
        has_authentication_key: arg_of(group, "authentication-key").is_some(),
        preempt: toggle(group, "preempt"),
        preempt_hold_time: child_block(group, "preempt")
            .and_then(|preempt| arg_of(preempt, "hold-time"))
            .map(str::to_string),
        virtual_addresses: ["virtual-address", "virtual-inet6-address"]
            .iter()
            .flat_map(|keyword| args_of(group, keyword))
            .flat_map(list_values)
            .collect(),
        virtual_link_local_address: field("virtual-link-local-address"),
        track: child_block(group, "track").map(parse_track),
        inherit_from: child_block(group, "vrrp-inherit-from").map(|inherit| VrrpInheritFrom {
            active_group: arg_of(inherit, "active-group").map(str::to_string),
            active_interface: arg_of(inherit, "active-interface").map(str::to_string),
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::interfaces::interfaces;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
interfaces {
    ge-0/0/0 {
        unit 0 {
            family inet {
                address 192.0.2.2/24 {
                    vrrp-group 10 {
                        virtual-address [ 192.0.2.1 192.0.2.254 ];
                        priority 200;
                        accept-data;
                        advertise-interval 2;
                        advertisements-threshold 3;
                        fast-interval 500;
                        preferred;
                        authentication-type md5;
                        authentication-key "$9$secret"; ## SECRET-DATA
                        preempt {
                            hold-time 60;
                        }
                        track {
                            interface ge-0/0/1 {
                                priority-cost 20;
                                bandwidth-threshold 1000000 priority-cost 5;
                                bandwidth-threshold 100000 priority-cost 10;
                            }
                            route 10.0.0.0/8 routing-instance default priority-cost 30;
                            priority-hold-time 15;
                        }
                        vrrp-inherit-from {
                            active-group 1;
                            active-interface ge-0/0/2.0;
                        }
                    }
                    vrrp-group 11 {
                        virtual-address 192.0.2.3;
                        no-accept-data;
                        no-preempt;
                    }
                }
            }
            family inet6 {
                address 2001:db8::2/64 {
                    vrrp-inet6-group 20 {
                        virtual-inet6-address [ 2001:db8::1 ];
                        virtual-link-local-address fe80::1;
                        inet6-advertise-interval 1000;
                        priority 150;
                        preempt;
                    }
                }
            }
        }
    }
}
"#;

    #[test]
    fn extracts_vrrp_group_without_the_key() {
        let ifaces = interfaces(&juniper::parse_braces(CONFIG));
        let groups = &ifaces[0].units[0].family_settings[0].addresses[0].vrrp_groups;
        assert_eq!(groups.len(), 2);

        let group = &groups[0];
        assert_eq!(group.group, "10");
        assert!(!group.inet6);
        assert_eq!(group.virtual_addresses, ["192.0.2.1", "192.0.2.254"]);
        assert_eq!(group.priority.as_deref(), Some("200"));
        assert_eq!(group.accept_data, Some(true));
        assert_eq!(group.advertise_interval.as_deref(), Some("2"));
        assert_eq!(group.advertisements_threshold.as_deref(), Some("3"));
        assert_eq!(group.fast_interval.as_deref(), Some("500"));
        assert!(group.preferred);
        assert_eq!(group.authentication_type.as_deref(), Some("md5"));
        assert!(group.has_authentication_key);
        assert_eq!(group.preempt, Some(true));
        assert_eq!(group.preempt_hold_time.as_deref(), Some("60"));

        let inherit = group.inherit_from.as_ref().unwrap();
        assert_eq!(inherit.active_group.as_deref(), Some("1"));
        assert_eq!(inherit.active_interface.as_deref(), Some("ge-0/0/2.0"));
    }

    #[test]
    fn extracts_vrrp_track_interfaces_and_routes() {
        let ifaces = interfaces(&juniper::parse_braces(CONFIG));
        let track = ifaces[0].units[0].family_settings[0].addresses[0].vrrp_groups[0]
            .track
            .as_ref()
            .unwrap();
        assert_eq!(track.priority_hold_time.as_deref(), Some("15"));

        let interface = &track.interfaces[0];
        assert_eq!(interface.name, "ge-0/0/1");
        assert_eq!(interface.priority_cost.as_deref(), Some("20"));
        assert_eq!(interface.bandwidth_thresholds.len(), 2);
        assert_eq!(interface.bandwidth_thresholds[0].bandwidth, "1000000");
        assert_eq!(interface.bandwidth_thresholds[0].priority_cost.as_deref(), Some("5"));

        let route = &track.routes[0];
        assert_eq!(route.prefix, "10.0.0.0/8");
        assert_eq!(route.routing_instance.as_deref(), Some("default"));
        assert_eq!(route.priority_cost.as_deref(), Some("30"));
    }

    #[test]
    fn minimal_vrrp_group_has_defaults() {
        let ifaces = interfaces(&juniper::parse_braces(CONFIG));
        let group = &ifaces[0].units[0].family_settings[0].addresses[0].vrrp_groups[1];
        assert_eq!(group.virtual_addresses, ["192.0.2.3"]);
        assert_eq!(group.accept_data, Some(false));
        assert_eq!(group.preempt, Some(false));
        assert_eq!(group.preempt_hold_time, None);
        assert!(!group.has_authentication_key);
        assert!(group.track.is_none());
        assert!(group.inherit_from.is_none());
    }

    #[test]
    fn extracts_vrrp_inet6_group() {
        let ifaces = interfaces(&juniper::parse_braces(CONFIG));
        let group = &ifaces[0].units[0].family_settings[1].addresses[0].vrrp_groups[0];
        assert_eq!(group.group, "20");
        assert!(group.inet6);
        assert_eq!(group.virtual_addresses, ["2001:db8::1"]);
        assert_eq!(group.virtual_link_local_address.as_deref(), Some("fe80::1"));
        assert_eq!(group.inet6_advertise_interval.as_deref(), Some("1000"));
        assert_eq!(group.preempt, Some(true));
    }

    #[test]
    fn address_without_vrrp_has_no_groups() {
        let ifaces = interfaces(&juniper::parse_braces(
            "interfaces {\n    ge-0/0/0 {\n        unit 0 {\n            family inet {\n                address 192.0.2.1/24;\n            }\n        }\n    }\n}\n",
        ));
        assert!(ifaces[0].units[0].family_settings[0].addresses[0]
            .vrrp_groups
            .is_empty());
    }
}
