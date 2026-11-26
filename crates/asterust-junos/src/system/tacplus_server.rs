//! Extracts the `tacplus-server` entries of `system`.
//!
//! Configuration path: `system { tacplus-server <address> { ... } }`.
//!
//! # See also
//!
//! - [tacplus-server](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/tacplus-server-edit-system.html)

use super::servers;
use crate::{arg_of, has_leaf};
use asterust::config::ConfigLine;

/// A `tacplus-server <address>`. The `secret` is not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TacplusServer {
    pub address: String,
    pub port: Option<String>,
    pub timeout: Option<String>,
    pub source_address: Option<String>,
    pub routing_instance: Option<String>,
    pub single_connection: bool,
}

pub(super) fn tacplus_servers(parent: &ConfigLine, keyword: &str) -> Vec<TacplusServer> {
    servers(parent, keyword)
        .into_iter()
        .map(|(address, server)| TacplusServer {
            address: address.to_string(),
            port: arg_of(server, "port").map(str::to_string),
            timeout: arg_of(server, "timeout").map(str::to_string),
            source_address: arg_of(server, "source-address").map(str::to_string),
            routing_instance: arg_of(server, "routing-instance").map(str::to_string),
            single_connection: has_leaf(server, "single-connection"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    #[test]
    fn extracts_tacplus_servers_without_secrets() {
        let system = system(&juniper::parse_braces(
            r#"
system {
    tacplus-server 192.0.2.20 {
        port 49;
        secret "$9$secret"; ## SECRET-DATA
        single-connection;
        source-address 192.0.2.1;
        timeout 5;
    }
    inactive: tacplus-server 192.0.2.21 {
        secret "$9$other"; ## SECRET-DATA
    }
}
"#,
        ))
        .unwrap();
        assert_eq!(system.tacplus_servers.len(), 1);
        let server = &system.tacplus_servers[0];
        assert_eq!(server.address, "192.0.2.20");
        assert_eq!(server.port.as_deref(), Some("49"));
        assert_eq!(server.timeout.as_deref(), Some("5"));
        assert_eq!(server.source_address.as_deref(), Some("192.0.2.1"));
        assert!(server.single_connection);
        assert_eq!(server.routing_instance, None);
    }
}
