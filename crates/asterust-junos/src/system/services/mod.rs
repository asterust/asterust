//! `system { services { ... } }`.

mod dhcp_local_server;
mod flow_tap_dtcp;
mod local_policy_decision_function;
mod netconf;
mod outbound_ssh;
mod service_deployment;
mod ssh;
mod static_subscribers;
mod subscriber_management;
mod web_management;
mod xnm_clear_text;
mod xnm_ssl;

pub use dhcp_local_server::{DhcpLocalServer, DhcpLocalServerGroup};
pub use flow_tap_dtcp::{FlowTapDtcp, FlowTapDtcpSsh};
pub use local_policy_decision_function::{AaclStatisticsProfile, LocalPolicyDecisionFunction};
pub use netconf::{Netconf, NetconfSsh};
pub use outbound_ssh::{OutboundSshClient, OutboundSshServer};
pub use service_deployment::{ServiceDeployment, ServiceDeploymentServer};
pub use ssh::SshService;
pub use static_subscribers::{StaticSubscriberGroup, StaticSubscriberProfile, StaticSubscribers};
pub use subscriber_management::{RedundancyInterface, SubscriberManagement, SubscriberRedundancy};
pub use web_management::{WebManagement, WebManagementHttp, WebManagementHttps};
pub use xnm_clear_text::XnmClearText;
pub use xnm_ssl::XnmSsl;

use crate::child_block;
use asterust::config::ConfigLine;
use dhcp_local_server::parse_dhcp_local_server;
use flow_tap_dtcp::parse_flow_tap_dtcp;
use local_policy_decision_function::parse_local_policy_decision_function;
use netconf::parse_netconf;
use outbound_ssh::parse_outbound_ssh;
use service_deployment::parse_service_deployment;
use ssh::parse_ssh;
use static_subscribers::parse_static_subscribers;
use subscriber_management::parse_subscriber_management;
use web_management::parse_web_management;
use xnm_clear_text::parse_xnm_clear_text;
use xnm_ssl::parse_xnm_ssl;

/// The `system services` configuration. Services other than those listed are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Services {
    pub ssh: Option<SshService>,
    pub telnet: bool,
    pub ftp: bool,
    /// `netconf { ssh; }`.
    pub netconf_ssh: bool,
    /// `web-management { http; }`.
    pub http: bool,
    /// `web-management { https; }`.
    pub https: bool,
    pub outbound_ssh: Vec<OutboundSshClient>,
    /// The details behind `http` and `https`.
    pub web_management: Option<WebManagement>,
    /// The details behind `netconf_ssh`.
    pub netconf: Option<Netconf>,
    pub xnm_clear_text: Option<XnmClearText>,
    pub xnm_ssl: Option<XnmSsl>,
    pub subscriber_management: Option<SubscriberManagement>,
    pub static_subscribers: Option<StaticSubscribers>,
    pub dhcp_local_server: Option<DhcpLocalServer>,
    pub local_policy_decision_function: Option<LocalPolicyDecisionFunction>,
    pub flow_tap_dtcp: Option<FlowTapDtcp>,
    pub service_deployment: Option<ServiceDeployment>,
}

pub(super) fn parse_services(services: &ConfigLine) -> Services {
    let web = child_block(services, "web-management");
    let netconf = child_block(services, "netconf");
    Services {
        ssh: child_block(services, "ssh").map(parse_ssh),
        telnet: child_block(services, "telnet").is_some(),
        ftp: child_block(services, "ftp").is_some(),
        netconf_ssh: netconf.is_some_and(|n| child_block(n, "ssh").is_some()),
        http: web.is_some_and(|w| child_block(w, "http").is_some()),
        https: web.is_some_and(|w| child_block(w, "https").is_some()),
        outbound_ssh: child_block(services, "outbound-ssh")
            .map(parse_outbound_ssh)
            .unwrap_or_default(),
        web_management: web.map(parse_web_management),
        netconf: netconf.map(parse_netconf),
        xnm_clear_text: child_block(services, "xnm-clear-text").map(parse_xnm_clear_text),
        xnm_ssl: child_block(services, "xnm-ssl").map(parse_xnm_ssl),
        subscriber_management: child_block(services, "subscriber-management").map(parse_subscriber_management),
        static_subscribers: child_block(services, "static-subscribers").map(parse_static_subscribers),
        dhcp_local_server: child_block(services, "dhcp-local-server").map(parse_dhcp_local_server),
        local_policy_decision_function: child_block(services, "local-policy-decision-function")
            .map(parse_local_policy_decision_function),
        flow_tap_dtcp: child_block(services, "flow-tap-dtcp").map(parse_flow_tap_dtcp),
        service_deployment: child_block(services, "service-deployment").map(parse_service_deployment),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::system;
    use asterust::config::juniper;

    const SERVICES: &str = r#"
system {
    services {
        ssh {
            root-login deny;
            protocol-version v2;
            port 2222;
            connection-limit 10;
            rate-limit 5;
            ciphers [ aes256-ctr aes128-ctr ];
            sftp-server;
        }
        netconf {
            ssh;
        }
        web-management {
            https {
                system-generated-certificate;
            }
        }
        telnet;
    }
}
"#;

    #[test]
    fn extracts_ssh_service_settings() {
        let services = system(&juniper::parse_braces(SERVICES)).unwrap().services.unwrap();
        let ssh = services.ssh.unwrap();
        assert_eq!(ssh.root_login.as_deref(), Some("deny"));
        assert_eq!(ssh.protocol_versions, ["v2"]);
        assert_eq!(ssh.port.as_deref(), Some("2222"));
        assert_eq!(ssh.connection_limit.as_deref(), Some("10"));
        assert_eq!(ssh.rate_limit.as_deref(), Some("5"));
        assert_eq!(ssh.ciphers, ["aes256-ctr", "aes128-ctr"]);
        assert!(ssh.sftp_server);
    }

    #[test]
    fn extracts_enabled_services() {
        let services = system(&juniper::parse_braces(SERVICES)).unwrap().services.unwrap();
        assert!(services.telnet);
        assert!(services.netconf_ssh);
        assert!(services.https);
        assert!(!services.http);
        assert!(!services.ftp);
    }

    #[test]
    fn bare_service_statements_enable_with_defaults() {
        let services = system(&juniper::parse_braces(
            "system {\n    services {\n        ssh;\n        ftp;\n        web-management {\n            http;\n        }\n    }\n}\n",
        ))
        .unwrap()
        .services
        .unwrap();
        assert_eq!(services.ssh, Some(SshService::default()));
        assert!(services.ftp);
        assert!(services.http);
        assert!(!services.telnet);
        assert!(!services.netconf_ssh);
    }

    #[test]
    fn disabled_service_is_not_reported() {
        let services = system(&juniper::parse_braces(
            "system {\n    services {\n        inactive: telnet;\n        ssh;\n    }\n}\n",
        ))
        .unwrap()
        .services
        .unwrap();
        assert!(!services.telnet);
    }

    #[test]
    fn extracts_netconf_and_xnm_settings() {
        let services = system(&juniper::parse_braces(
            r#"
system {
    services {
        netconf {
            ssh {
                port 830;
                connection-limit 5;
                rate-limit 10;
            }
            rfc-compliant;
        }
        xnm-clear-text {
            connection-limit 3;
        }
        xnm-ssl {
            port 3220;
            local-certificate XNM;
            rate-limit 7;
        }
    }
}
"#,
        ))
        .unwrap()
        .services
        .unwrap();
        let netconf = services.netconf.unwrap();
        assert!(netconf.rfc_compliant);
        let ssh = netconf.ssh.unwrap();
        assert_eq!(ssh.port.as_deref(), Some("830"));
        assert_eq!(ssh.connection_limit.as_deref(), Some("5"));
        assert_eq!(ssh.rate_limit.as_deref(), Some("10"));
        assert!(services.netconf_ssh);

        let clear_text = services.xnm_clear_text.unwrap();
        assert_eq!(clear_text.connection_limit.as_deref(), Some("3"));
        assert_eq!(clear_text.rate_limit, None);

        let ssl = services.xnm_ssl.unwrap();
        assert_eq!(ssl.port.as_deref(), Some("3220"));
        assert_eq!(ssl.local_certificate.as_deref(), Some("XNM"));
        assert_eq!(ssl.rate_limit.as_deref(), Some("7"));
    }

    #[test]
    fn no_services_is_none() {
        assert!(system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n"))
            .unwrap()
            .services
            .is_none());
    }

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
