//! `address`
//! `interfaces { <name> { unit <n> { family <name> { address <address> { ... } } } } }`.
//!
//! # See also
//!
//! - [address (IP Address)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/address-edit-interfaces-unit-family.html)
//! - [primary (Address on Interface)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/primary-edit-interfaces.html)
//! - [preferred](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/preferred-edit-interfaces.html)
//! - [arp](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/arp-edit-interfaces.html)
//! - [destination (Tunnels)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/destination-edit-interfaces.html)
//! - [master-only](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/master-only-edit-interfaces.html)

mod vrrp_group;

pub use vrrp_group::{BandwidthThreshold, TrackedInterface, TrackedRoute, VrrpGroup, VrrpInheritFrom, VrrpTrack};

use crate::{active_children, has_leaf, split_name};
use asterust::config::ConfigLine;
use vrrp_group::parse_vrrp_groups;

/// A static `arp` entry of an address.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AddressArp {
    pub address: String,
    pub mac: Option<String>,
    /// The MAC is written with `multicast-mac`.
    pub multicast_mac: bool,
    /// The device also answers ARP requests for the address.
    pub publish: bool,
}

/// An `address <address>` of a family, with its options.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnitAddress {
    /// e.g. `"10.0.0.1/24"`.
    pub address: String,
    pub primary: bool,
    pub preferred: bool,
    pub master_only: bool,
    pub eui_64: bool,
    pub destination: Option<String>,
    pub broadcast: Option<String>,
    pub arps: Vec<AddressArp>,
    pub vrrp_groups: Vec<VrrpGroup>,
}

pub(super) fn parse_addresses(family: &ConfigLine) -> Vec<UnitAddress> {
    active_children(family)
        .filter_map(|line| Some((line, line.arg("address")?)))
        .map(|(line, arg)| parse_address(line, arg))
        .collect()
}

fn parse_address(line: &ConfigLine, arg: &str) -> UnitAddress {
    let (address, inline) = split_name(arg);
    let inline: Vec<&str> = inline.split_whitespace().collect();
    let flag = |name: &str| has_leaf(line, name) || inline.contains(&name);
    let value = |keyword: &str| {
        active_children(line)
            .find_map(|c| c.arg(keyword))
            .map(|arg| arg.trim_matches('"').to_string())
    };
    UnitAddress {
        address: address.to_string(),
        primary: flag("primary"),
        preferred: flag("preferred"),
        master_only: flag("master-only"),
        eui_64: flag("eui-64"),
        destination: value("destination"),
        broadcast: value("broadcast"),
        arps: active_children(line)
            .filter_map(|c| c.arg("arp"))
            .map(parse_arp)
            .collect(),
        vrrp_groups: parse_vrrp_groups(line),
    }
}

/// `<ip-address> (mac | multicast-mac) <mac-address> [publish]`.
fn parse_arp(arg: &str) -> AddressArp {
    let mut words = arg.split_whitespace();
    let address = words.next().unwrap_or_default().to_string();
    let mut arp = AddressArp {
        address,
        mac: None,
        multicast_mac: false,
        publish: false,
    };
    while let Some(word) = words.next() {
        match word {
            "mac" => arp.mac = words.next().map(str::to_string),
            "multicast-mac" => {
                arp.multicast_mac = true;
                arp.mac = words.next().map(str::to_string);
            }
            "publish" => arp.publish = true,
            _ => {}
        }
    }
    arp
}
