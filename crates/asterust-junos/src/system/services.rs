//! `system { services { ... } }`.

use crate::{active_children, arg_of, args_of, child_block, has_leaf, list_values, named_children};
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
    SshService {
        root_login: arg_of(ssh, "root-login").map(str::to_string),
        protocol_versions: list("protocol-version"),
        port: arg_of(ssh, "port").map(str::to_string),
        connection_limit: arg_of(ssh, "connection-limit").map(str::to_string),
        rate_limit: arg_of(ssh, "rate-limit").map(str::to_string),
        ciphers: list("ciphers"),
        sftp_server: has_leaf(ssh, "sftp-server"),
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
    fn no_services_is_none() {
        assert!(system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n"))
            .unwrap()
            .services
            .is_none());
    }
}
