//! `subscriber-management`
//! `system { services { subscriber-management { ... } } }`.
//!
//! # See also
//!
//! - [subscriber-management](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/subscriber-management-edit-system-services.html)

use crate::{active_children, arg_of, child_block, has_leaf, named_children};
use asterust::config::ConfigLine;

/// An `interface <name>` of a subscriber-management `redundancy` block. The `shared-key` is not
/// extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct RedundancyInterface {
    pub name: String,
    pub local_inet_address: Option<String>,
    pub local_inet6_address: Option<String>,
    pub virtual_inet_address: Option<String>,
    pub virtual_inet6_address: Option<String>,
}

/// The `redundancy` block of `subscriber-management`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SubscriberRedundancy {
    pub interfaces: Vec<RedundancyInterface>,
    pub no_advertise_routes_on_backup: bool,
    /// `pseudo-wire` and `vrrp`.
    pub protocols: Vec<String>,
}

/// The `system services subscriber-management` settings. `overrides` and `traceoptions` are not
/// extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SubscriberManagement {
    pub enable: bool,
    /// `cgnat` is configured.
    pub cgnat: bool,
    pub enforce_strict_scale_limit_license: bool,
    pub gres_route_flush_delay: Option<String>,
    pub redundancy: Option<SubscriberRedundancy>,
}

pub(super) fn parse_subscriber_management(management: &ConfigLine) -> SubscriberManagement {
    SubscriberManagement {
        enable: has_leaf(management, "enable"),
        cgnat: child_block(management, "cgnat").is_some(),
        enforce_strict_scale_limit_license: has_leaf(management, "enforce-strict-scale-limit-license"),
        gres_route_flush_delay: arg_of(management, "gres-route-flush-delay").map(str::to_string),
        redundancy: child_block(management, "redundancy").map(|redundancy| SubscriberRedundancy {
            interfaces: named_children(redundancy, "interface")
                .map(|(name, interface)| RedundancyInterface {
                    name: name.to_string(),
                    local_inet_address: arg_of(interface, "local-inet-address").map(str::to_string),
                    local_inet6_address: arg_of(interface, "local-inet6-address").map(str::to_string),
                    virtual_inet_address: arg_of(interface, "virtual-inet-address").map(str::to_string),
                    virtual_inet6_address: arg_of(interface, "virtual-inet6-address").map(str::to_string),
                })
                .collect(),
            no_advertise_routes_on_backup: has_leaf(redundancy, "no-advertise-routes-on-backup"),
            protocols: child_block(redundancy, "protocol")
                .map(|protocol| active_children(protocol).map(|p| p.text.clone()).collect())
                .unwrap_or_default(),
        }),
    }
}
