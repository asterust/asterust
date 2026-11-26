//! `static-subscribers`
//! `system { services { static-subscribers { ... } } }`.
//!
//! # See also
//!
//! - [static-subscribers (Dynamic Service Provisioning)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/static-subscribers-edit-system-services.html)

use crate::{arg_of, args_of, named_children};
use asterust::config::ConfigLine;

/// A `dynamic-profile <name>` of a static-subscribers group.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct StaticSubscriberProfile {
    pub name: String,
    /// `merge` or `replace`.
    pub aggregate_clients: Option<String>,
}

/// A `group <name>` under `static-subscribers`. Authentication settings are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct StaticSubscriberGroup {
    pub name: String,
    pub access_profile: Option<String>,
    pub dynamic_profiles: Vec<StaticSubscriberProfile>,
    /// Each `interface` as written, e.g. `"ge-1/0/0.100 upto ge-1/0/0.200"` or `"ge-1/0/0.5
    /// exclude"`.
    pub interfaces: Vec<String>,
}

/// The `system services static-subscribers` settings. Authentication settings, which include a
/// password, are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct StaticSubscribers {
    pub access_profile: Option<String>,
    pub service_profile: Option<String>,
    pub groups: Vec<StaticSubscriberGroup>,
}

pub(super) fn parse_static_subscribers(subscribers: &ConfigLine) -> StaticSubscribers {
    StaticSubscribers {
        access_profile: arg_of(subscribers, "access-profile").map(str::to_string),
        service_profile: arg_of(subscribers, "service-profile").map(str::to_string),
        groups: named_children(subscribers, "group")
            .map(|(name, group)| StaticSubscriberGroup {
                name: name.to_string(),
                access_profile: arg_of(group, "access-profile").map(str::to_string),
                dynamic_profiles: named_children(group, "dynamic-profile")
                    .map(|(name, profile)| StaticSubscriberProfile {
                        name: name.to_string(),
                        aggregate_clients: arg_of(profile, "aggregate-clients").map(str::to_string),
                    })
                    .collect(),
                interfaces: args_of(group, "interface").into_iter().map(str::to_string).collect(),
            })
            .collect(),
    }
}
