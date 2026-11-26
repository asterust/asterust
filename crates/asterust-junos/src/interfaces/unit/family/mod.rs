//! `family`
//! `interfaces { <name> { unit <n> { family <name> { ... } } } }`.
//!
//! # See also
//!
//! - [family (interfaces)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/family-edit-interfaces.html)
//! - [mtu (interfaces)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/mtu-edit-interfaces-ni.html)
//! - [vlan-id (Logical Port in Bridge Domain)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/vlan-id-edit-interfaces-family-bridge.html)
//! - [vlan-id-list (Interface in Bridge Domain)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/vlan-id-list-edit-interfaces.html)

mod address;
mod filter;
mod policer;
mod rpf_check;
mod sampling;

pub use address::{
    AddressArp, BandwidthThreshold, TrackedInterface, TrackedRoute, UnitAddress, VrrpGroup, VrrpInheritFrom, VrrpTrack,
};
pub use filter::FamilyFilter;
pub use policer::FamilyPolicer;
pub use rpf_check::RpfCheck;
pub use sampling::FamilySampling;

use crate::{arg_of, args_of, list_values};
use address::parse_addresses;
use asterust::config::ConfigLine;
use filter::parse_filter;
use policer::parse_policer;
use rpf_check::parse_rpf_check;
use sampling::parse_sampling;

/// A `family <name>` of a logical unit, with the settings that apply to it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnitFamily {
    /// e.g. `"inet"`, `"inet6"`, `"iso"`, `"mpls"`.
    pub name: String,
    pub mtu: Option<String>,
    pub filter: Option<FamilyFilter>,
    pub rpf_check: Option<RpfCheck>,
    pub policer: Option<FamilyPolicer>,
    pub sampling: Option<FamilySampling>,
    /// `vlan-id` of `family bridge`.
    pub vlan_id: Option<String>,
    /// `vlan-id-list` of `family bridge`: IDs and `<from>-<to>` ranges.
    pub vlan_id_list: Vec<String>,
    pub addresses: Vec<UnitAddress>,
}

pub(super) fn parse_family(name: &str, family: &ConfigLine) -> UnitFamily {
    UnitFamily {
        name: name.to_string(),
        mtu: arg_of(family, "mtu").map(str::to_string),
        filter: parse_filter(family),
        rpf_check: parse_rpf_check(family),
        policer: parse_policer(family),
        sampling: parse_sampling(family),
        vlan_id: arg_of(family, "vlan-id").map(str::to_string),
        vlan_id_list: args_of(family, "vlan-id-list")
            .into_iter()
            .flat_map(list_values)
            .collect(),
        addresses: parse_addresses(family),
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
                mtu 1500;
                filter {
                    group 5;
                    input PROTECT-IN;
                    output PROTECT-OUT;
                    input-list [ A B ];
                    output-list C;
                }
                rpf-check {
                    fail-filter RPF-FAIL;
                    mode loose;
                }
                policer {
                    arp ARP-POLICER;
                    input IN-POLICER;
                    output OUT-POLICER;
                }
                sampling {
                    input;
                    output;
                }
                address 192.0.2.1/24 {
                    primary;
                    preferred;
                    destination 192.0.2.2;
                    broadcast 192.0.2.255;
                    arp 192.0.2.50 mac 00:11:22:33:44:55 publish;
                    arp 192.0.2.51 multicast-mac 01:00:5e:00:00:01;
                }
                address 192.0.2.9/24;
            }
            family inet6 {
                rpf-check;
                sampling input;
                address 2001:db8::1/64 {
                    eui-64;
                }
            }
            family iso;
        }
    }
}
"#;

    #[test]
    fn extracts_family_filters_policers_and_sampling() {
        let ifaces = interfaces(&juniper::parse_braces(CONFIG));
        let inet = &ifaces[0].units[0].family_settings[0];
        assert_eq!(inet.name, "inet");
        assert_eq!(inet.mtu.as_deref(), Some("1500"));

        let filter = inet.filter.as_ref().unwrap();
        assert_eq!(filter.group.as_deref(), Some("5"));
        assert_eq!(filter.input.as_deref(), Some("PROTECT-IN"));
        assert_eq!(filter.output.as_deref(), Some("PROTECT-OUT"));
        assert_eq!(filter.input_list, ["A", "B"]);
        assert_eq!(filter.output_list, ["C"]);

        let policer = inet.policer.as_ref().unwrap();
        assert_eq!(policer.arp.as_deref(), Some("ARP-POLICER"));
        assert_eq!(policer.input.as_deref(), Some("IN-POLICER"));
        assert_eq!(policer.output.as_deref(), Some("OUT-POLICER"));

        let sampling = inet.sampling.as_ref().unwrap();
        assert!(sampling.input);
        assert!(sampling.output);
    }

    #[test]
    fn extracts_rpf_check_in_block_and_flag_form() {
        let ifaces = interfaces(&juniper::parse_braces(CONFIG));
        let families = &ifaces[0].units[0].family_settings;

        let rpf = families[0].rpf_check.as_ref().unwrap();
        assert_eq!(rpf.fail_filter.as_deref(), Some("RPF-FAIL"));
        assert_eq!(rpf.mode.as_deref(), Some("loose"));

        let bare = families[1].rpf_check.as_ref().unwrap();
        assert_eq!(bare.fail_filter, None);
        assert_eq!(bare.mode, None);
    }

    #[test]
    fn extracts_address_options_and_arp_entries() {
        let ifaces = interfaces(&juniper::parse_braces(CONFIG));
        let addresses = &ifaces[0].units[0].family_settings[0].addresses;
        assert_eq!(addresses.len(), 2);

        let first = &addresses[0];
        assert_eq!(first.address, "192.0.2.1/24");
        assert!(first.primary);
        assert!(first.preferred);
        assert!(!first.master_only);
        assert_eq!(first.destination.as_deref(), Some("192.0.2.2"));
        assert_eq!(first.broadcast.as_deref(), Some("192.0.2.255"));
        assert_eq!(first.arps.len(), 2);
        assert_eq!(first.arps[0].address, "192.0.2.50");
        assert_eq!(first.arps[0].mac.as_deref(), Some("00:11:22:33:44:55"));
        assert!(first.arps[0].publish);
        assert!(!first.arps[0].multicast_mac);
        assert!(first.arps[1].multicast_mac);
        assert!(!first.arps[1].publish);

        let second = &addresses[1];
        assert_eq!(second.address, "192.0.2.9/24");
        assert!(!second.primary);
        assert!(second.arps.is_empty());
    }

    #[test]
    fn extracts_inline_sampling_and_eui_64() {
        let ifaces = interfaces(&juniper::parse_braces(CONFIG));
        let inet6 = &ifaces[0].units[0].family_settings[1];
        let sampling = inet6.sampling.as_ref().unwrap();
        assert!(sampling.input);
        assert!(!sampling.output);
        assert!(inet6.addresses[0].eui_64);
    }

    #[test]
    fn family_without_settings_has_none() {
        let ifaces = interfaces(&juniper::parse_braces(CONFIG));
        let iso = &ifaces[0].units[0].family_settings[2];
        assert_eq!(iso.name, "iso");
        assert!(iso.mtu.is_none());
        assert!(iso.filter.is_none());
        assert!(iso.rpf_check.is_none());
        assert!(iso.policer.is_none());
        assert!(iso.sampling.is_none());
        assert!(iso.addresses.is_empty());
    }
}
