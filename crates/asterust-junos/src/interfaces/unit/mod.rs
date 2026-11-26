//! `interfaces { <name> { unit <n> { ... } } }`.

mod family;
mod vlan_map;
mod vlan_tags;

pub use family::{
    AddressArp, BandwidthThreshold, FamilyFilter, FamilyPolicer, FamilySampling, RpfCheck, TrackedInterface,
    TrackedRoute, UnitAddress, UnitFamily, VrrpGroup, VrrpInheritFrom, VrrpTrack,
};
pub use vlan_map::VlanMap;
pub use vlan_tags::VlanTags;

use crate::{arg_of, args_of, named_children};
use asterust::config::ConfigLine;
use family::parse_family;
use vlan_map::parse_vlan_map;
use vlan_tags::parse_vlan_tags;

/// A logical `unit <n>` of an interface. Addresses are the raw `address` values (e.g. `"10.0.0.1/24"`;
/// IS-IS NETs for `family iso`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct InterfaceUnit {
    pub number: String,
    pub description: Option<String>,
    pub vlan_id: Option<u32>,
    /// `vlan-id unmatched`.
    pub vlan_id_unmatched: bool,
    /// e.g. `"100-200"`.
    pub vlan_id_range: Option<String>,
    pub vlan_tags: Option<VlanTags>,
    pub input_vlan_map: Option<VlanMap>,
    pub output_vlan_map: Option<VlanMap>,
    /// Logical encapsulation, e.g. `"vlan-bridge"` or `"vlan-ccc"`.
    pub encapsulation: Option<String>,
    /// Every `family <name>` present, e.g. `"inet"`, `"inet6"`, `"iso"`, `"mpls"`.
    pub families: Vec<String>,
    pub inet_addresses: Vec<String>,
    pub inet6_addresses: Vec<String>,
    pub iso_addresses: Vec<String>,
    /// The settings of each `family`, in the order written.
    pub family_settings: Vec<UnitFamily>,
}

pub(super) fn parse_unit(number: &str, unit: &ConfigLine) -> InterfaceUnit {
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
        vlan_id_unmatched: arg_of(unit, "vlan-id") == Some("unmatched"),
        vlan_id_range: arg_of(unit, "vlan-id-range").map(str::to_string),
        vlan_tags: parse_vlan_tags(unit),
        input_vlan_map: parse_vlan_map(unit, "input-vlan-map"),
        output_vlan_map: parse_vlan_map(unit, "output-vlan-map"),
        encapsulation: arg_of(unit, "encapsulation").map(str::to_string),
        inet_addresses: addresses("inet"),
        inet6_addresses: addresses("inet6"),
        iso_addresses: addresses("iso"),
        families: families.iter().map(|(name, _)| name.to_string()).collect(),
        family_settings: families
            .iter()
            .map(|(name, family)| parse_family(name, family))
            .collect(),
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
        flexible-vlan-tagging;
        encapsulation flexible-ethernet-services;
        unit 100 {
            encapsulation vlan-bridge;
            vlan-id 100;
            input-vlan-map {
                swap;
                tag-protocol-id 0x8100;
                vlan-id 200;
            }
            output-vlan-map swap;
        }
        unit 200 {
            vlan-tags outer 0x8100.200 inner 0x8100.300;
            input-vlan-map {
                push-push;
                tag-protocol-id 0x9100;
                vlan-id 10;
                inner-tag-protocol-id 0x8100;
                inner-vlan-id 20;
            }
            output-vlan-map {
                pop-pop;
            }
        }
        unit 300 {
            vlan-id unmatched;
        }
        unit 400 {
            vlan-id-range 400-499;
            vlan-tags outer 500 inner-range 1-4094;
        }
    }
}
"#;

    #[test]
    fn extracts_unit_vlan_id_forms_and_encapsulation() {
        let units = &interfaces(&juniper::parse_braces(CONFIG))[0].units;
        assert_eq!(units[0].vlan_id, Some(100));
        assert!(!units[0].vlan_id_unmatched);
        assert_eq!(units[0].encapsulation.as_deref(), Some("vlan-bridge"));

        assert_eq!(units[2].vlan_id, None);
        assert!(units[2].vlan_id_unmatched);

        assert_eq!(units[3].vlan_id_range.as_deref(), Some("400-499"));
        assert_eq!(units[1].encapsulation, None);
    }

    #[test]
    fn extracts_vlan_tags() {
        let units = &interfaces(&juniper::parse_braces(CONFIG))[0].units;
        let tags = units[1].vlan_tags.as_ref().unwrap();
        assert_eq!(tags.outer.as_deref(), Some("0x8100.200"));
        assert_eq!(tags.inner.as_deref(), Some("0x8100.300"));
        assert_eq!(tags.inner_range, None);

        let ranged = units[3].vlan_tags.as_ref().unwrap();
        assert_eq!(ranged.outer.as_deref(), Some("500"));
        assert_eq!(ranged.inner, None);
        assert_eq!(ranged.inner_range.as_deref(), Some("1-4094"));

        assert!(units[0].vlan_tags.is_none());
    }

    #[test]
    fn extracts_vlan_maps_in_block_and_one_line_form() {
        let units = &interfaces(&juniper::parse_braces(CONFIG))[0].units;

        let input = units[0].input_vlan_map.as_ref().unwrap();
        assert_eq!(input.operation.as_deref(), Some("swap"));
        assert_eq!(input.tag_protocol_id.as_deref(), Some("0x8100"));
        assert_eq!(input.vlan_id.as_deref(), Some("200"));
        assert_eq!(input.inner_vlan_id, None);
        assert_eq!(
            units[0].output_vlan_map.as_ref().unwrap().operation.as_deref(),
            Some("swap")
        );

        let input = units[1].input_vlan_map.as_ref().unwrap();
        assert_eq!(input.operation.as_deref(), Some("push-push"));
        assert_eq!(input.tag_protocol_id.as_deref(), Some("0x9100"));
        assert_eq!(input.vlan_id.as_deref(), Some("10"));
        assert_eq!(input.inner_tag_protocol_id.as_deref(), Some("0x8100"));
        assert_eq!(input.inner_vlan_id.as_deref(), Some("20"));
        assert_eq!(
            units[1].output_vlan_map.as_ref().unwrap().operation.as_deref(),
            Some("pop-pop")
        );

        assert!(units[2].input_vlan_map.is_none());
        assert!(units[2].output_vlan_map.is_none());
    }

    #[test]
    fn extracts_bridge_family_vlan_ids() {
        let ifaces = interfaces(&juniper::parse_braces(
            "interfaces {\n    ge-0/0/1 {\n        unit 0 {\n            family bridge {\n                vlan-id-list [ 100-200 300 ];\n                vlan-id 50;\n            }\n        }\n    }\n}\n",
        ));
        let bridge = &ifaces[0].units[0].family_settings[0];
        assert_eq!(bridge.name, "bridge");
        assert_eq!(bridge.vlan_id_list, ["100-200", "300"]);
        assert_eq!(bridge.vlan_id.as_deref(), Some("50"));
    }
}
