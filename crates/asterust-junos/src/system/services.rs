//! `system { services { ... } }`.

use super::flow_tap_dtcp::{parse_flow_tap_dtcp, FlowTapDtcp};
use super::service_deployment::{parse_service_deployment, ServiceDeployment};
use super::subscriber::{
    parse_dhcp_local_server, parse_local_policy_decision_function, parse_static_subscribers,
    parse_subscriber_management, DhcpLocalServer, LocalPolicyDecisionFunction, StaticSubscribers, SubscriberManagement,
};
use crate::{
    active_children, arg_of, args_of, child_block, has_leaf, list_values, named_children, statement_words, value_after,
};
use asterust::config::ConfigLine;

/// The `system services ssh` settings; every field is `None` or empty when only `ssh;` is written.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct SshService {
    /// `allow`, `deny` or `deny-password`.
    pub root_login: Option<String>,
    pub protocol_versions: Vec<String>,
    pub port: Option<String>,
    pub connection_limit: Option<String>,
    pub rate_limit: Option<String>,
    pub ciphers: Vec<String>,
    pub sftp_server: bool,
    /// Methods tried in order: `password`, `radius`, `tacplus`.
    pub authentication_order: Vec<String>,
    pub key_exchange: Vec<String>,
    pub macs: Vec<String>,
    pub client_alive_count_max: Option<String>,
    pub client_alive_interval: Option<String>,
    /// `md5` or `sha2-256`.
    pub fingerprint_hash: Option<String>,
    pub max_sessions_per_connection: Option<String>,
    pub max_pre_authentication_packets: Option<String>,
    pub no_challenge_response: bool,
    pub no_password_authentication: bool,
    pub no_passwords: bool,
    pub no_public_keys: bool,
}

/// An `address` of an outbound-ssh client.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OutboundSshServer {
    pub address: String,
    pub port: Option<String>,
    pub retry: Option<String>,
    pub timeout: Option<String>,
}

/// An `outbound-ssh client <id>`. The `secret` is not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OutboundSshClient {
    pub id: String,
    pub device_id: Option<String>,
    pub servers: Vec<OutboundSshServer>,
    /// e.g. `"netconf"`.
    pub services: Vec<String>,
    /// `in-order` or `sticky`.
    pub reconnect_strategy: Option<String>,
    pub routing_instance: Option<String>,
}

/// The `web-management http` block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct WebManagementHttp {
    pub interfaces: Vec<String>,
    pub port: Option<String>,
}

/// The `web-management https` block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct WebManagementHttps {
    pub interfaces: Vec<String>,
    pub port: Option<String>,
    pub local_certificate: Option<String>,
    pub pki_local_certificate: Option<String>,
    pub system_generated_certificate: bool,
}

/// The `system services web-management` settings.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct WebManagement {
    /// From `control max-threads`.
    pub control_max_threads: Option<String>,
    pub http: Option<WebManagementHttp>,
    pub https: Option<WebManagementHttps>,
    pub management_url: Option<String>,
    /// Minutes, from `session idle-timeout`.
    pub session_idle_timeout: Option<String>,
    pub session_limit: Option<String>,
}

/// The `netconf ssh` block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct NetconfSsh {
    pub port: Option<String>,
    pub connection_limit: Option<String>,
    pub rate_limit: Option<String>,
}

/// The `system services netconf` settings.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Netconf {
    pub ssh: Option<NetconfSsh>,
    pub rfc_compliant: bool,
}

/// The `xnm-clear-text` block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct XnmClearText {
    pub connection_limit: Option<String>,
    pub rate_limit: Option<String>,
}

/// The `xnm-ssl` block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct XnmSsl {
    pub port: Option<String>,
    pub connection_limit: Option<String>,
    pub rate_limit: Option<String>,
    pub local_certificate: Option<String>,
}

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
            .map(|outbound| {
                named_children(outbound, "client")
                    .map(|(id, client)| parse_outbound_client(id, client))
                    .collect()
            })
            .unwrap_or_default(),
        web_management: web.map(parse_web_management),
        netconf: netconf.map(|netconf| Netconf {
            ssh: child_block(netconf, "ssh").map(|ssh| NetconfSsh {
                port: arg_of(ssh, "port").map(str::to_string),
                connection_limit: arg_of(ssh, "connection-limit").map(str::to_string),
                rate_limit: arg_of(ssh, "rate-limit").map(str::to_string),
            }),
            rfc_compliant: has_leaf(netconf, "rfc-compliant"),
        }),
        xnm_clear_text: child_block(services, "xnm-clear-text").map(|xnm| XnmClearText {
            connection_limit: arg_of(xnm, "connection-limit").map(str::to_string),
            rate_limit: arg_of(xnm, "rate-limit").map(str::to_string),
        }),
        xnm_ssl: child_block(services, "xnm-ssl").map(|xnm| XnmSsl {
            port: arg_of(xnm, "port").map(str::to_string),
            connection_limit: arg_of(xnm, "connection-limit").map(str::to_string),
            rate_limit: arg_of(xnm, "rate-limit").map(str::to_string),
            local_certificate: arg_of(xnm, "local-certificate").map(str::to_string),
        }),
        subscriber_management: child_block(services, "subscriber-management").map(parse_subscriber_management),
        static_subscribers: child_block(services, "static-subscribers").map(parse_static_subscribers),
        dhcp_local_server: child_block(services, "dhcp-local-server").map(parse_dhcp_local_server),
        local_policy_decision_function: child_block(services, "local-policy-decision-function")
            .map(parse_local_policy_decision_function),
        flow_tap_dtcp: child_block(services, "flow-tap-dtcp").map(parse_flow_tap_dtcp),
        service_deployment: child_block(services, "service-deployment").map(parse_service_deployment),
    }
}

fn parse_web_management(web: &ConfigLine) -> WebManagement {
    let interfaces =
        |block: &ConfigLine| -> Vec<String> { args_of(block, "interface").into_iter().flat_map(list_values).collect() };
    let session = child_block(web, "session");
    WebManagement {
        control_max_threads: statement_words(web, "control").and_then(|words| value_after(&words, "max-threads")),
        http: child_block(web, "http").map(|http| WebManagementHttp {
            interfaces: interfaces(http),
            port: arg_of(http, "port").map(str::to_string),
        }),
        https: child_block(web, "https").map(|https| WebManagementHttps {
            interfaces: interfaces(https),
            port: arg_of(https, "port").map(str::to_string),
            local_certificate: arg_of(https, "local-certificate").map(str::to_string),
            pki_local_certificate: arg_of(https, "pki-local-certificate").map(str::to_string),
            system_generated_certificate: has_leaf(https, "system-generated-certificate"),
        }),
        management_url: arg_of(web, "management-url").map(str::to_string),
        session_idle_timeout: session.and_then(|s| arg_of(s, "idle-timeout")).map(str::to_string),
        session_limit: session.and_then(|s| arg_of(s, "session-limit")).map(str::to_string),
    }
}

fn parse_outbound_client(id: &str, client: &ConfigLine) -> OutboundSshClient {
    OutboundSshClient {
        id: id.to_string(),
        device_id: arg_of(client, "device-id").map(str::to_string),
        servers: active_children(client)
            // Servers are the children written as a bare address; the single-word blocks are other options.
            .filter(|c| !c.text.contains(' ') && !matches!(c.text.as_str(), "keep-alive" | "proxy-server"))
            .map(|server| OutboundSshServer {
                address: server.text.clone(),
                port: arg_of(server, "port").map(str::to_string),
                retry: arg_of(server, "retry").map(str::to_string),
                timeout: arg_of(server, "timeout").map(str::to_string),
            })
            .collect(),
        services: args_of(client, "services").into_iter().flat_map(list_values).collect(),
        reconnect_strategy: arg_of(client, "reconnect-strategy").map(str::to_string),
        routing_instance: arg_of(client, "routing-instance").map(str::to_string),
    }
}

fn parse_ssh(ssh: &ConfigLine) -> SshService {
    let list = |keyword: &str| args_of(ssh, keyword).into_iter().flat_map(list_values).collect();
    let field = |keyword: &str| arg_of(ssh, keyword).map(str::to_string);
    SshService {
        root_login: field("root-login"),
        protocol_versions: list("protocol-version"),
        port: field("port"),
        connection_limit: field("connection-limit"),
        rate_limit: field("rate-limit"),
        ciphers: list("ciphers"),
        sftp_server: has_leaf(ssh, "sftp-server"),
        authentication_order: list("authentication-order"),
        key_exchange: list("key-exchange"),
        macs: list("macs"),
        client_alive_count_max: field("client-alive-count-max"),
        client_alive_interval: field("client-alive-interval"),
        fingerprint_hash: field("fingerprint-hash"),
        max_sessions_per_connection: field("max-sessions-per-connection"),
        max_pre_authentication_packets: field("max-pre-authentication-packets"),
        no_challenge_response: has_leaf(ssh, "no-challenge-response"),
        no_password_authentication: has_leaf(ssh, "no-password-authentication"),
        no_passwords: has_leaf(ssh, "no-passwords"),
        no_public_keys: has_leaf(ssh, "no-public-keys"),
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
    fn extracts_outbound_ssh_clients() {
        let services = system(&juniper::parse_braces(
            r#"
system {
    services {
        outbound-ssh {
            client mgmt {
                device-id pe1;
                secret "$9$secret"; ## SECRET-DATA
                reconnect-strategy sticky;
                routing-instance mgmt_junos;
                services netconf;
                192.0.2.50 {
                    port 7804;
                    retry 5;
                    timeout 20;
                }
            }
            client backup {
                192.0.2.60;
                keep-alive {
                    retry 3;
                    timeout 15;
                }
            }
        }
    }
}
"#,
        ))
        .unwrap()
        .services
        .unwrap();
        assert_eq!(services.outbound_ssh.len(), 2);

        let mgmt = &services.outbound_ssh[0];
        assert_eq!(mgmt.id, "mgmt");
        assert_eq!(mgmt.device_id.as_deref(), Some("pe1"));
        assert_eq!(mgmt.reconnect_strategy.as_deref(), Some("sticky"));
        assert_eq!(mgmt.routing_instance.as_deref(), Some("mgmt_junos"));
        assert_eq!(mgmt.services, ["netconf"]);
        assert_eq!(mgmt.servers.len(), 1);
        assert_eq!(mgmt.servers[0].address, "192.0.2.50");
        assert_eq!(mgmt.servers[0].port.as_deref(), Some("7804"));
        assert_eq!(mgmt.servers[0].retry.as_deref(), Some("5"));
        assert_eq!(mgmt.servers[0].timeout.as_deref(), Some("20"));

        let backup = &services.outbound_ssh[1];
        assert_eq!(backup.id, "backup");
        // `keep-alive` is an option block, not a server.
        assert_eq!(backup.servers.len(), 1);
        assert_eq!(backup.servers[0].address, "192.0.2.60");
        assert_eq!(backup.servers[0].port, None);
    }

    #[test]
    fn no_outbound_ssh_is_empty() {
        let services = system(&juniper::parse_braces(
            "system {\n    services {\n        ssh;\n    }\n}\n",
        ))
        .unwrap()
        .services
        .unwrap();
        assert!(services.outbound_ssh.is_empty());
    }

    #[test]
    fn extracts_more_ssh_settings() {
        let ssh = system(&juniper::parse_braces(
            r#"
system {
    services {
        ssh {
            authentication-order [ tacplus password ];
            key-exchange [ curve25519-sha256 ecdh-sha2-nistp256 ];
            macs [ hmac-sha2-256 hmac-sha2-512 ];
            client-alive-count-max 3;
            client-alive-interval 60;
            fingerprint-hash sha2-256;
            max-sessions-per-connection 5;
            max-pre-authentication-packets 64;
            no-challenge-response;
            no-password-authentication;
            no-public-keys;
        }
    }
}
"#,
        ))
        .unwrap()
        .services
        .unwrap()
        .ssh
        .unwrap();
        assert_eq!(ssh.authentication_order, ["tacplus", "password"]);
        assert_eq!(ssh.key_exchange, ["curve25519-sha256", "ecdh-sha2-nistp256"]);
        assert_eq!(ssh.macs, ["hmac-sha2-256", "hmac-sha2-512"]);
        assert_eq!(ssh.client_alive_count_max.as_deref(), Some("3"));
        assert_eq!(ssh.client_alive_interval.as_deref(), Some("60"));
        assert_eq!(ssh.fingerprint_hash.as_deref(), Some("sha2-256"));
        assert_eq!(ssh.max_sessions_per_connection.as_deref(), Some("5"));
        assert_eq!(ssh.max_pre_authentication_packets.as_deref(), Some("64"));
        assert!(ssh.no_challenge_response);
        assert!(ssh.no_password_authentication);
        assert!(ssh.no_public_keys);
        assert!(!ssh.no_passwords);
    }

    #[test]
    fn extracts_web_management_details() {
        let web = system(&juniper::parse_braces(
            r#"
system {
    services {
        web-management {
            control {
                max-threads 8;
            }
            management-url /manage;
            http {
                interface [ fxp0.0 ge-0/0/1.0 ];
                port 8080;
            }
            https {
                interface fxp0.0;
                port 8443;
                pki-local-certificate J-WEB;
            }
            session {
                idle-timeout 20;
                session-limit 4;
            }
        }
    }
}
"#,
        ))
        .unwrap()
        .services
        .unwrap()
        .web_management
        .unwrap();
        assert_eq!(web.control_max_threads.as_deref(), Some("8"));
        assert_eq!(web.management_url.as_deref(), Some("/manage"));
        assert_eq!(web.session_idle_timeout.as_deref(), Some("20"));
        assert_eq!(web.session_limit.as_deref(), Some("4"));

        let http = web.http.unwrap();
        assert_eq!(http.interfaces, ["fxp0.0", "ge-0/0/1.0"]);
        assert_eq!(http.port.as_deref(), Some("8080"));

        let https = web.https.unwrap();
        assert_eq!(https.interfaces, ["fxp0.0"]);
        assert_eq!(https.port.as_deref(), Some("8443"));
        assert_eq!(https.pki_local_certificate.as_deref(), Some("J-WEB"));
        assert_eq!(https.local_certificate, None);
        assert!(!https.system_generated_certificate);
    }

    #[test]
    fn web_management_with_system_generated_certificate() {
        let web = system(&juniper::parse_braces(
            "system {\n    services {\n        web-management {\n            https {\n                system-generated-certificate;\n            }\n        }\n    }\n}\n",
        ))
        .unwrap()
        .services
        .unwrap()
        .web_management
        .unwrap();
        assert!(web.https.unwrap().system_generated_certificate);
        assert!(web.http.is_none());
        assert_eq!(web.control_max_threads, None);
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
    fn bare_netconf_ssh_has_no_settings() {
        let netconf = system(&juniper::parse_braces(
            "system {\n    services {\n        netconf {\n            ssh;\n        }\n    }\n}\n",
        ))
        .unwrap()
        .services
        .unwrap()
        .netconf
        .unwrap();
        let ssh = netconf.ssh.unwrap();
        assert_eq!(ssh.port, None);
        assert!(!netconf.rfc_compliant);
    }

    #[test]
    fn no_services_is_none() {
        assert!(system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n"))
            .unwrap()
            .services
            .is_none());
    }
}
