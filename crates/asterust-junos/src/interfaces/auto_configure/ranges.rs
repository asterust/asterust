//! `vlan-ranges` and `stacked-vlan-ranges`
//! `interfaces { <name> { auto-configure { vlan-ranges { ... } stacked-vlan-ranges { ... } } } }`.
//!
//! # See also
//!
//! - [vlan-ranges](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/vlan-ranges-edit-interfaces.html)
//! - [stacked-vlan-ranges](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/stacked-vlan-ranges-edit-interfaces.html)
//! - [dynamic-profile (VLAN)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/dynamic-profile-edit-interfaces-vlan-ranges.html)

use crate::{arg_of, args_of, child_block, named_children};
use asterust::config::ConfigLine;

/// A `dynamic-profile <name>` of a VLAN range block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AutoConfigureProfile {
    pub name: String,
    /// Each `accept` statement, e.g. `"dhcp-v4"` or `"pppoe"`.
    pub accept: Vec<String>,
    pub access_profile: Option<String>,
    /// e.g. `"100-200"`, or for stacked ranges `"any,100-200"`.
    pub ranges: Option<String>,
}

/// A `vlan-ranges` or `stacked-vlan-ranges` block. The `authentication` password is not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AutoConfigureRanges {
    pub access_profile: Option<String>,
    pub dynamic_profiles: Vec<AutoConfigureProfile>,
    /// An `authentication` block is configured.
    pub has_authentication: bool,
}

pub(super) fn parse_ranges(ranges: &ConfigLine) -> AutoConfigureRanges {
    AutoConfigureRanges {
        access_profile: arg_of(ranges, "access-profile").map(str::to_string),
        dynamic_profiles: named_children(ranges, "dynamic-profile")
            .map(|(name, profile)| AutoConfigureProfile {
                name: name.to_string(),
                accept: args_of(profile, "accept").into_iter().map(str::to_string).collect(),
                access_profile: arg_of(profile, "access-profile").map(str::to_string),
                ranges: arg_of(profile, "ranges").map(str::to_string),
            })
            .collect(),
        has_authentication: child_block(ranges, "authentication").is_some(),
    }
}
