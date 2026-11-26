//! `system { radius-server <address> { ... } }`.

use super::servers;
use crate::arg_of;
use asterust::config::ConfigLine;

/// A `radius-server <address>`. The `secret` is not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct RadiusServer {
    pub address: String,
    pub port: Option<String>,
    pub accounting_port: Option<String>,
    pub retry: Option<String>,
    pub timeout: Option<String>,
    pub source_address: Option<String>,
    pub routing_instance: Option<String>,
}

/// The servers under `parent` written as `<keyword> ...`: `radius-server` in `system`, `server` in accounting.
pub(super) fn radius_servers(parent: &ConfigLine, keyword: &str) -> Vec<RadiusServer> {
    servers(parent, keyword)
        .into_iter()
        .map(|(address, server)| RadiusServer {
            address: address.to_string(),
            port: arg_of(server, "port").map(str::to_string),
            accounting_port: arg_of(server, "accounting-port").map(str::to_string),
            retry: arg_of(server, "retry").map(str::to_string),
            timeout: arg_of(server, "timeout").map(str::to_string),
            source_address: arg_of(server, "source-address").map(str::to_string),
            routing_instance: arg_of(server, "routing-instance").map(str::to_string),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    #[test]
    fn extracts_radius_servers_without_secrets() {
        let system = system(&juniper::parse_braces(
            r#"
system {
    radius-server 192.0.2.10 {
        secret "$9$secret"; ## SECRET-DATA
        port 1812;
        accounting-port 1813;
        retry 5;
        timeout 10;
        source-address 192.0.2.1;
        routing-instance mgmt_junos;
    }
    radius-server 192.0.2.11 {
        secret "$9$other"; ## SECRET-DATA
    }
}
"#,
        ))
        .unwrap();
        assert_eq!(system.radius_servers.len(), 2);

        let first = &system.radius_servers[0];
        assert_eq!(first.address, "192.0.2.10");
        assert_eq!(first.port.as_deref(), Some("1812"));
        assert_eq!(first.accounting_port.as_deref(), Some("1813"));
        assert_eq!(first.retry.as_deref(), Some("5"));
        assert_eq!(first.timeout.as_deref(), Some("10"));
        assert_eq!(first.source_address.as_deref(), Some("192.0.2.1"));
        assert_eq!(first.routing_instance.as_deref(), Some("mgmt_junos"));

        assert_eq!(system.radius_servers[1].address, "192.0.2.11");
        assert_eq!(system.radius_servers[1].port, None);
    }

    #[test]
    fn extracts_radius_servers_in_nested_block_form() {
        let system = system(&juniper::parse_braces(
            "system {\n    radius-server {\n        192.0.2.10 {\n            port 1645;\n        }\n    }\n}\n",
        ))
        .unwrap();
        assert_eq!(system.radius_servers[0].address, "192.0.2.10");
        assert_eq!(system.radius_servers[0].port.as_deref(), Some("1645"));
    }
}
