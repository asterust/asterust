//! `interfaces { <name> { unit <n> { ... } } }`.

mod family;

pub use family::{
    AddressArp, BandwidthThreshold, FamilyFilter, FamilyPolicer, FamilySampling, RpfCheck, TrackedInterface,
    TrackedRoute, UnitAddress, UnitFamily, VrrpGroup, VrrpInheritFrom, VrrpTrack,
};

use crate::{arg_of, args_of, named_children};
use asterust::config::ConfigLine;
use family::parse_family;

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
