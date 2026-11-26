//! Subscriber-management settings under `system`: `services subscriber-management`, `static-subscribers`,
//! `dhcp-local-server`, `local-policy-decision-function` and `dynamic-profile-options`.

use crate::{active_children, arg_of, args_of, child_block, has_leaf, named_children};
use asterust::config::ConfigLine;

/// An `interface <name>` of a subscriber-management `redundancy` block. The `shared-key` is not extracted.
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

/// The `system services subscriber-management` settings. `overrides` and `traceoptions` are not extracted.
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
    /// Each `interface` as written, e.g. `"ge-1/0/0.100 upto ge-1/0/0.200"` or `"ge-1/0/0.5 exclude"`.
    pub interfaces: Vec<String>,
}

/// The `system services static-subscribers` settings. Authentication settings, which include a password, are not
/// extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct StaticSubscribers {
    pub access_profile: Option<String>,
    pub service_profile: Option<String>,
    pub groups: Vec<StaticSubscriberGroup>,
}

/// A `group <name>` of `dhcp-local-server`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DhcpLocalServerGroup {
    pub name: String,
    /// Each `interface` as written.
    pub interfaces: Vec<String>,
}

/// The `system services dhcp-local-server` groups. The many per-group options are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DhcpLocalServer {
    pub groups: Vec<DhcpLocalServerGroup>,
    /// Groups under `dhcpv6`.
    pub dhcpv6_groups: Vec<DhcpLocalServerGroup>,
}

/// An `aacl-statistics-profile <name>`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AaclStatisticsProfile {
    pub name: String,
    pub aacl_fields: Vec<String>,
    pub file: Option<String>,
    /// `interim-active-only` or `interim-full`.
    pub record_mode: Option<String>,
    /// Minutes.
    pub report_interval: Option<String>,
}

/// The `system services local-policy-decision-function` settings.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LocalPolicyDecisionFunction {
    /// Profiles from `statistics`.
    pub statistics: Vec<AaclStatisticsProfile>,
}

/// The `system dynamic-profile-options` settings.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DynamicProfileOptions {
    pub versioning: bool,
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

pub(super) fn parse_dhcp_local_server(server: &ConfigLine) -> DhcpLocalServer {
    let groups = |parent: &ConfigLine| -> Vec<DhcpLocalServerGroup> {
        named_children(parent, "group")
            .map(|(name, group)| DhcpLocalServerGroup {
                name: name.to_string(),
                interfaces: args_of(group, "interface").into_iter().map(str::to_string).collect(),
            })
            .collect()
    };
    DhcpLocalServer {
        groups: groups(server),
        dhcpv6_groups: child_block(server, "dhcpv6").map(groups).unwrap_or_default(),
    }
}

pub(super) fn parse_local_policy_decision_function(function: &ConfigLine) -> LocalPolicyDecisionFunction {
    LocalPolicyDecisionFunction {
        statistics: child_block(function, "statistics")
            .map(|statistics| {
                named_children(statistics, "aacl-statistics-profile")
                    .map(|(name, profile)| AaclStatisticsProfile {
                        name: name.to_string(),
                        aacl_fields: child_block(profile, "aacl-fields")
                            .map(|fields| active_children(fields).map(|f| f.text.clone()).collect())
                            .unwrap_or_default(),
                        file: arg_of(profile, "file").map(str::to_string),
                        record_mode: arg_of(profile, "record-mode").map(str::to_string),
                        report_interval: arg_of(profile, "report-interval").map(str::to_string),
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

pub(super) fn parse_dynamic_profile_options(options: &ConfigLine) -> DynamicProfileOptions {
    DynamicProfileOptions {
        versioning: has_leaf(options, "versioning"),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const SUBSCRIBERS: &str = r#"
system {
    dynamic-profile-options {
        versioning;
    }
    services {
        subscriber-management {
            enable;
            enforce-strict-scale-limit-license;
            gres-route-flush-delay 60;
            cgnat {
                enable;
            }
            redundancy {
                interface ae0.100 {
                    local-inet-address 192.0.2.1;
                    local-inet6-address 2001:db8::1;
                    shared-key "$9$secret"; ## SECRET-DATA
                    virtual-inet-address 192.0.2.254;
                    virtual-inet6-address 2001:db8::fe;
                }
                no-advertise-routes-on-backup;
                protocol {
                    vrrp;
                }
            }
        }
        static-subscribers {
            access-profile AP;
            service-profile SP;
            authentication {
                password "$9$secret"; ## SECRET-DATA
            }
            group STATIC1 {
                access-profile AP1;
                authentication {
                    password "$9$other"; ## SECRET-DATA
                }
                dynamic-profile PROFILE1 {
                    aggregate-clients merge;
                }
                interface ge-1/0/0.100 upto ge-1/0/0.200;
                interface ge-1/0/0.150 exclude;
            }
        }
        dhcp-local-server {
            group G1 {
                interface ge-1/0/0.0;
                interface ge-1/0/1.0 upto ge-1/0/1.9;
            }
            dhcpv6 {
                group V6 {
                    interface ge-2/0/0.0;
                }
            }
        }
        local-policy-decision-function {
            statistics {
                aacl-statistics-profile AACL1 {
                    aacl-fields {
                        input-bytes;
                        output-bytes;
                    }
                    file aacl.stats;
                    record-mode interim-full;
                    report-interval 30;
                }
            }
        }
    }
}
"#;

    #[test]
    fn extracts_subscriber_management() {
        let system = system(&juniper::parse_braces(SUBSCRIBERS)).unwrap();
        let management = system.services.unwrap().subscriber_management.unwrap();
        assert!(management.enable);
        assert!(management.cgnat);
        assert!(management.enforce_strict_scale_limit_license);
        assert_eq!(management.gres_route_flush_delay.as_deref(), Some("60"));

        let redundancy = management.redundancy.unwrap();
        assert!(redundancy.no_advertise_routes_on_backup);
        assert_eq!(redundancy.protocols, ["vrrp"]);
        let interface = &redundancy.interfaces[0];
        assert_eq!(interface.name, "ae0.100");
        assert_eq!(interface.local_inet_address.as_deref(), Some("192.0.2.1"));
        assert_eq!(interface.local_inet6_address.as_deref(), Some("2001:db8::1"));
        assert_eq!(interface.virtual_inet_address.as_deref(), Some("192.0.2.254"));
        assert_eq!(interface.virtual_inet6_address.as_deref(), Some("2001:db8::fe"));
    }

    #[test]
    fn extracts_static_subscribers() {
        let system = system(&juniper::parse_braces(SUBSCRIBERS)).unwrap();
        let subscribers = system.services.unwrap().static_subscribers.unwrap();
        assert_eq!(subscribers.access_profile.as_deref(), Some("AP"));
        assert_eq!(subscribers.service_profile.as_deref(), Some("SP"));

        let group = &subscribers.groups[0];
        assert_eq!(group.name, "STATIC1");
        assert_eq!(group.access_profile.as_deref(), Some("AP1"));
        assert_eq!(group.dynamic_profiles[0].name, "PROFILE1");
        assert_eq!(group.dynamic_profiles[0].aggregate_clients.as_deref(), Some("merge"));
        assert_eq!(
            group.interfaces,
            ["ge-1/0/0.100 upto ge-1/0/0.200", "ge-1/0/0.150 exclude"]
        );
    }

    #[test]
    fn extracts_dhcp_local_server_groups() {
        let system = system(&juniper::parse_braces(SUBSCRIBERS)).unwrap();
        let server = system.services.unwrap().dhcp_local_server.unwrap();
        assert_eq!(server.groups.len(), 1);
        assert_eq!(server.groups[0].name, "G1");
        assert_eq!(
            server.groups[0].interfaces,
            ["ge-1/0/0.0", "ge-1/0/1.0 upto ge-1/0/1.9"]
        );
        assert_eq!(server.dhcpv6_groups[0].name, "V6");
        assert_eq!(server.dhcpv6_groups[0].interfaces, ["ge-2/0/0.0"]);
    }

    #[test]
    fn extracts_aacl_statistics_profiles() {
        let system = system(&juniper::parse_braces(SUBSCRIBERS)).unwrap();
        let function = system.services.unwrap().local_policy_decision_function.unwrap();
        let profile = &function.statistics[0];
        assert_eq!(profile.name, "AACL1");
        assert_eq!(profile.aacl_fields, ["input-bytes", "output-bytes"]);
        assert_eq!(profile.file.as_deref(), Some("aacl.stats"));
        assert_eq!(profile.record_mode.as_deref(), Some("interim-full"));
        assert_eq!(profile.report_interval.as_deref(), Some("30"));
    }

    #[test]
    fn extracts_dynamic_profile_options() {
        let system = system(&juniper::parse_braces(SUBSCRIBERS)).unwrap();
        assert!(system.dynamic_profile_options.unwrap().versioning);
    }

    #[test]
    fn subscriber_settings_are_optional() {
        let system = system(&juniper::parse_braces(
            "system {\n    services {\n        ssh;\n    }\n}\n",
        ))
        .unwrap();
        assert!(system.dynamic_profile_options.is_none());
        let services = system.services.unwrap();
        assert!(services.subscriber_management.is_none());
        assert!(services.static_subscribers.is_none());
        assert!(services.dhcp_local_server.is_none());
        assert!(services.local_policy_decision_function.is_none());
    }
}
