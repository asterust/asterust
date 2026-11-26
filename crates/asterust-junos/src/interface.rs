//! `interfaces { ... }`: physical interfaces and their logical units.

use crate::{active_children, arg_of, args_of, has_leaf, named_children, top_level};
use asterust::config::{Config, ConfigLine};

/// A logical `unit <n>` of an interface. Addresses are the raw `address` values (e.g. `"10.0.0.1/24"`;
/// IS-IS NETs for `family iso`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct InterfaceUnit {
    pub number: String,
    pub description: Option<String>,
    pub vlan_id: Option<u32>,
    /// Every `family <name>` present, e.g. `"inet"`, `"inet6"`, `"iso"`, `"mpls"`.
    pub families: Vec<String>,
    pub inet_addresses: Vec<String>,
    pub inet6_addresses: Vec<String>,
    pub iso_addresses: Vec<String>,
}

/// A physical or loopback interface such as `ge-0/0/0` or `lo0`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Interface {
    pub name: String,
    pub description: Option<String>,
    /// Set by the `disable` statement.
    pub disabled: bool,
    pub units: Vec<InterfaceUnit>,
}

/// Every interface under the top-level `interfaces` block.
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
        units: named_children(line, "unit")
            .map(|(number, unit)| parse_unit(number, unit))
            .collect(),
    }
}

fn parse_unit(number: &str, unit: &ConfigLine) -> InterfaceUnit {
    let families: Vec<_> = named_children(unit, "family").collect();
    let addresses = |family: &str| -> Vec<String> {
        families
            .iter()
            .filter(|(name, _)| *name == family)
            .flat_map(|(_, f)| args_of(f, "address"))
            .map(str::to_string)
            .collect()
    };
    InterfaceUnit {
        number: number.to_string(),
        description: arg_of(unit, "description").map(str::to_string),
        vlan_id: arg_of(unit, "vlan-id").and_then(|v| v.parse().ok()),
        inet_addresses: addresses("inet"),
        inet6_addresses: addresses("inet6"),
        iso_addresses: addresses("iso"),
        families: families.iter().map(|(name, _)| name.to_string()).collect(),
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
