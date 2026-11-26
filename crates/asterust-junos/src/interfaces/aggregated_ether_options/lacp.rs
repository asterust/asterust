//! `lacp`
//! `interfaces { <name> { aggregated-ether-options { lacp { ... } } } }`.
//!
//! # See also
//!
//! - [lacp (Aggregated Ethernet)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/lacp-edit-interfaces-aggregated-ethernet-qfx-series.html)

use crate::{arg_of, has_leaf};
use asterust::config::ConfigLine;

/// The `lacp` settings of an aggregated Ethernet bundle.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Lacp {
    /// `active` or `passive`; LACP is passive when neither is written.
    pub mode: Option<String>,
    /// `fast` or `slow`.
    pub periodic: Option<String>,
    pub system_priority: Option<String>,
    pub system_id: Option<String>,
    pub admin_key: Option<String>,
    pub force_up: bool,
    pub fast_failover: bool,
    pub accept_data: bool,
}

pub(super) fn parse_lacp(lacp: &ConfigLine) -> Lacp {
    Lacp {
        mode: ["active", "passive"]
            .iter()
            .find(|mode| has_leaf(lacp, mode))
            .map(|mode| mode.to_string()),
        periodic: arg_of(lacp, "periodic").map(str::to_string),
        system_priority: arg_of(lacp, "system-priority").map(str::to_string),
        system_id: arg_of(lacp, "system-id").map(str::to_string),
        admin_key: arg_of(lacp, "admin-key").map(str::to_string),
        force_up: has_leaf(lacp, "force-up"),
        fast_failover: has_leaf(lacp, "fast-failover"),
        accept_data: has_leaf(lacp, "accept-data"),
    }
}
