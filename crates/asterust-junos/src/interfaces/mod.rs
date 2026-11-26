//! `interfaces`
//! `interfaces { ... }`.
//!
//! # See also
//!
//! - [`[edit interfaces]` hierarchy](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/topic-map/hierarchy-edit-interfaces.html)
//! - [mtu (interfaces)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/mtu-edit-interfaces-ni.html)
//! - [speed (Ethernet)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/speed-edit-interfaces-ethernet.html)
//! - [encapsulation (interfaces)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/encapsulation-edit-interfaces-physical.html)
//! - [vlan-tagging](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/vlan-tagging-edit-interfaces.html)
//! - [flexible-vlan-tagging](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/flexible-vlan-tagging-edit-interfaces.html)
//! - [native-vlan-id](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/native-vlan-id-edit-interfaces-qfx-series.html)
//! - [per-unit-scheduler](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/per-unit-scheduler-edit-interfaces.html)
//! - [no-gratuitous-arp-request](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/no-gratuitous-arp-request-edit-interfaces-port-security.html)

mod aggregated_ether_options;
mod auto_configure;
mod ether_options;
mod hold_time;
mod link_degrade_monitor;
mod optics_options;
mod otn_options;
mod unit;

pub use aggregated_ether_options::{AggregatedEtherOptions, BfdLivenessDetection, Lacp};
pub use auto_configure::{AutoConfigure, AutoConfigureProfile, AutoConfigureRanges};
pub use ether_options::{EtherOptions, Ieee8023ad};
pub use hold_time::HoldTime;
pub use link_degrade_monitor::{LinkDegradeMonitor, LinkDegradeThresholds};
pub use optics_options::{OpticsLowLight, OpticsOptions};
pub use otn_options::{OtnOptions, OtnRate, SignalDegrade};
pub use unit::{
    AddressArp, BandwidthThreshold, FamilyFilter, FamilyPolicer, FamilySampling, InterfaceUnit, RpfCheck,
    TrackedInterface, TrackedRoute, UnitAddress, UnitFamily, VlanMap, VlanTags, VrrpGroup, VrrpInheritFrom, VrrpTrack,
};

use crate::{active_children, arg_of, child_block, has_leaf, named_children, top_level};
use aggregated_ether_options::parse_aggregated_ether_options;
use asterust::config::{Config, ConfigLine};
use auto_configure::parse_auto_configure;
use ether_options::parse_ether_options;
use hold_time::parse_hold_time;
use link_degrade_monitor::parse_link_degrade_monitor;
use optics_options::parse_optics_options;
use otn_options::parse_otn_options;
use unit::parse_unit;

/// A physical or loopback interface such as `ge-0/0/0` or `lo0`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Interface {
    pub name: String,
    pub description: Option<String>,
    /// Set by the `disable` statement.
    pub disabled: bool,
    pub mtu: Option<String>,
    /// e.g. `"1g"` or `"auto"`.
    pub speed: Option<String>,
    /// Physical encapsulation, e.g. `"flexible-ethernet-services"`.
    pub encapsulation: Option<String>,
    pub vlan_tagging: bool,
    pub flexible_vlan_tagging: bool,
    pub native_vlan_id: Option<String>,
    pub per_unit_scheduler: bool,
    pub no_gratuitous_arp_request: bool,
    pub hold_time: Option<HoldTime>,
    pub ether_options: Option<EtherOptions>,
    pub aggregated_ether_options: Option<AggregatedEtherOptions>,
    pub auto_configure: Option<AutoConfigure>,
    pub optics_options: Option<OpticsOptions>,
    pub otn_options: Option<OtnOptions>,
    pub link_degrade_monitor: Option<LinkDegradeMonitor>,
    pub units: Vec<InterfaceUnit>,
}

/// Returns every interface under the top-level `interfaces` block.
///
/// # Examples
///
/// ```
/// use asterust::config::juniper;
/// use asterust_junos::interfaces::interfaces;
///
/// let config = juniper::parse_braces("interfaces {\n    ge-0/0/0 {\n        mtu 9192;\n    }\n}\n");
/// let found = interfaces(&config);
/// assert_eq!(found[0].name, "ge-0/0/0");
/// assert_eq!(found[0].mtu.as_deref(), Some("9192"));
/// ```
pub fn interfaces(config: &Config) -> Vec<Interface> {
    let Some(root) = top_level(config, "interfaces") else {
        return Vec::new();
    };
    // Interface names have no spaces; this skips `apply-groups`, `interface-range <x>` and similar.
    active_children(root)
        .filter(|c| !c.text.contains(' '))
        .map(parse_interface)
        .collect()
}

fn parse_interface(line: &ConfigLine) -> Interface {
    Interface {
        name: line.text.clone(),
        description: arg_of(line, "description").map(str::to_string),
        disabled: has_leaf(line, "disable"),
        mtu: arg_of(line, "mtu").map(str::to_string),
        speed: arg_of(line, "speed").map(str::to_string),
        encapsulation: arg_of(line, "encapsulation").map(str::to_string),
        vlan_tagging: has_leaf(line, "vlan-tagging"),
        flexible_vlan_tagging: has_leaf(line, "flexible-vlan-tagging"),
        native_vlan_id: arg_of(line, "native-vlan-id").map(str::to_string),
        per_unit_scheduler: has_leaf(line, "per-unit-scheduler"),
        no_gratuitous_arp_request: has_leaf(line, "no-gratuitous-arp-request"),
        hold_time: parse_hold_time(line),
        ether_options: parse_ether_options(line),
        aggregated_ether_options: child_block(line, "aggregated-ether-options").map(parse_aggregated_ether_options),
        auto_configure: child_block(line, "auto-configure").map(parse_auto_configure),
        optics_options: child_block(line, "optics-options").map(parse_optics_options),
        otn_options: child_block(line, "otn-options").map(parse_otn_options),
        link_degrade_monitor: child_block(line, "link-degrade-monitor").map(parse_link_degrade_monitor),
        units: named_children(line, "unit")
            .map(|(number, unit)| parse_unit(number, unit))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::juniper;

    // Not from documentation: covers statements the Juniper examples do not use. See the README "Sources".
    const CONFIG: &str = r#"
interfaces {
    ge-0/0/0 {
        description "to p1";
        unit 0 {
            description "core link";
            family inet {
                address 10.0.1.1/30;
                address 10.0.2.1/30 {
                    primary;
                }
            }
            family inet6 {
                address 2001:db8:1::1/64;
            }
            family iso;
            family mpls;
        }
    }
    ge-0/0/1 {
        disable;
        vlan-tagging;
        unit 100 {
            vlan-id 100;
            family inet {
                address 192.0.2.1/24;
            }
        }
    }
    inactive: ge-0/0/2 {
        unit 0;
    }
    lo0 {
        unit 0 {
            family inet {
                address 10.255.0.1/32;
            }
            family iso {
                address 49.0001.0100.2550.0001.00;
            }
        }
    }
    apply-groups GLOBAL;
}
"#;

    #[test]
    fn extracts_units_families_and_addresses() {
        let ifaces = interfaces(&juniper::parse_braces(CONFIG));
        let names: Vec<_> = ifaces.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, ["ge-0/0/0", "ge-0/0/1", "lo0"]);

        let core = &ifaces[0];
        assert_eq!(core.description.as_deref(), Some("to p1"));
        assert!(!core.disabled);
        let unit = &core.units[0];
        assert_eq!(unit.number, "0");
        assert_eq!(unit.description.as_deref(), Some("core link"));
        assert_eq!(unit.families, ["inet", "inet6", "iso", "mpls"]);
        assert_eq!(unit.inet_addresses, ["10.0.1.1/30", "10.0.2.1/30"]);
        assert_eq!(unit.inet6_addresses, ["2001:db8:1::1/64"]);
        assert!(unit.iso_addresses.is_empty());
    }

    #[test]
    fn disabled_vlan_units_and_loopback_net() {
        let ifaces = interfaces(&juniper::parse_braces(CONFIG));
        assert!(ifaces[1].disabled);
        assert_eq!(ifaces[1].units[0].number, "100");
        assert_eq!(ifaces[1].units[0].vlan_id, Some(100));
        assert_eq!(ifaces[2].units[0].iso_addresses, ["49.0001.0100.2550.0001.00"]);
    }

    #[test]
    fn missing_interfaces_block_is_empty() {
        assert!(interfaces(&juniper::parse_braces("system {\n host-name r1;\n}\n")).is_empty());
    }

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_physical_interface_settings() {
        let ifaces = interfaces(&juniper::parse_braces(
            r#"
interfaces {
    xe-0/0/0 {
        mtu 9192;
        speed 10g;
        encapsulation flexible-ethernet-services;
        vlan-tagging;
        flexible-vlan-tagging;
        native-vlan-id 10;
        per-unit-scheduler;
        no-gratuitous-arp-request;
    }
    inactive: xe-0/0/1 {
        mtu 1500;
    }
    xe-0/0/2 {
        inactive: mtu 1500;
    }
}
"#,
        ));
        assert_eq!(ifaces.len(), 2);

        let xe0 = &ifaces[0];
        assert_eq!(xe0.mtu.as_deref(), Some("9192"));
        assert_eq!(xe0.speed.as_deref(), Some("10g"));
        assert_eq!(xe0.encapsulation.as_deref(), Some("flexible-ethernet-services"));
        assert!(xe0.vlan_tagging);
        assert!(xe0.flexible_vlan_tagging);
        assert_eq!(xe0.native_vlan_id.as_deref(), Some("10"));
        assert!(xe0.per_unit_scheduler);
        assert!(xe0.no_gratuitous_arp_request);

        assert_eq!(ifaces[1].mtu, None);
    }

    #[test]
    fn physical_interface_settings_default_to_off() {
        let ifaces = interfaces(&juniper::parse_braces(
            "interfaces {\n    ge-0/0/0 {\n        disable;\n    }\n}\n",
        ));
        let ge = &ifaces[0];
        assert_eq!(ge.mtu, None);
        assert_eq!(ge.speed, None);
        assert_eq!(ge.encapsulation, None);
        assert!(!ge.vlan_tagging);
        assert!(!ge.flexible_vlan_tagging);
        assert_eq!(ge.native_vlan_id, None);
        assert!(!ge.per_unit_scheduler);
        assert!(!ge.no_gratuitous_arp_request);
    }

    // The DOC_* fixtures are adapted from Juniper's BGP User Guide, "BGP Peering Sessions"; see the README "Sources".

    // "Example: Configuring External BGP Point-to-Point Peer Sessions", Device E `show interfaces` (two of four interfaces).
    const DOC_POINT_TO_POINT: &str = r#"
interfaces {
    ge-1/2/0 {
        unit 0 {
            description to-A;
            family inet {
                address 10.10.10.1/30;
            }
        }
    }
    ge-0/0/1 {
        unit 5 {
            description to-B;
            family inet {
                address 10.10.10.5/30;
            }
        }
    }
}
"#;

    // "Example: Configure Multiple Single-Hop EBGP Sessions on Different Links Using the Same IPv6 Link-Local
    // Address", R1 `show interfaces`.
    const DOC_LINK_LOCAL: &str = r#"
interfaces {
    ge-0/0/1 {
        description R1-to-R2-Link;
        vlan-tagging;
        unit 1 {
            vlan-id 1;
            family inet6 {
                address fe80::10/64;
            }
        }
        unit 2 {
            vlan-id 2;
            family inet6 {
                address fe80::10/64;
            }
        }
    }
    lo0 {
        unit 0 {
            family inet {
                address 198.51.100.1/24 {
                    primary;
                }
            }
        }
    }
}
"#;

    #[test]
    fn doc_point_to_point_interfaces() {
        let ifaces = interfaces(&juniper::parse_braces(DOC_POINT_TO_POINT));
        let names: Vec<_> = ifaces.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, ["ge-1/2/0", "ge-0/0/1"]);
        assert_eq!(ifaces[0].units[0].number, "0");
        assert_eq!(ifaces[0].units[0].description.as_deref(), Some("to-A"));
        assert_eq!(ifaces[0].units[0].inet_addresses, ["10.10.10.1/30"]);
        assert_eq!(ifaces[1].units[0].number, "5");
        assert_eq!(ifaces[1].units[0].inet_addresses, ["10.10.10.5/30"]);
    }

    #[test]
    fn doc_link_local_units_vlans_and_primary_address() {
        let ifaces = interfaces(&juniper::parse_braces(DOC_LINK_LOCAL));
        let ge = &ifaces[0];
        assert_eq!(ge.description.as_deref(), Some("R1-to-R2-Link"));
        assert_eq!(ge.units.len(), 2);
        assert_eq!(ge.units[0].vlan_id, Some(1));
        assert_eq!(ge.units[1].vlan_id, Some(2));
        assert_eq!(ge.units[0].inet6_addresses, ["fe80::10/64"]);

        let lo0 = &ifaces[1];
        assert_eq!(lo0.units[0].inet_addresses, ["198.51.100.1/24"]);
    }
}
