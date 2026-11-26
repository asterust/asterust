//! `outbound-ssh`
//! `system { services { outbound-ssh { client <id> { ... } } } }`.
//!
//! # See also
//!
//! - [outbound-ssh](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/outbound-ssh-edit-system.html)

use crate::{active_children, arg_of, args_of, list_values, named_children};
use asterust::config::ConfigLine;

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

pub(super) fn parse_outbound_ssh(outbound: &ConfigLine) -> Vec<OutboundSshClient> {
    named_children(outbound, "client")
        .map(|(id, client)| parse_outbound_client(id, client))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

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
}
