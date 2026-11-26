//! `system { ... }`.

use crate::{active_children, arg_of, top_level, unquote};
use asterust::config::{Config, ConfigLine};

/// A DNS server from `name-server`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct NameServer {
    pub address: String,
    /// Set when the server is reached through a routing instance such as `mgmt_junos`.
    pub routing_instance: Option<String>,
}

/// The top-level `system` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct System {
    pub host_name: Option<String>,
    pub name_servers: Vec<NameServer>,
}

/// The top-level `system` block, or `None` if there is none.
pub fn system(config: &Config) -> Option<System> {
    let system = top_level(config, "system")?;
    Some(System {
        host_name: arg_of(system, "host-name").map(str::to_string),
        name_servers: name_servers(system),
    })
}

/// Servers written as a `name-server { <address>; ... }` block, or one per `name-server <address>` line.
fn name_servers(system: &ConfigLine) -> Vec<NameServer> {
    let mut found = Vec::new();
    for child in active_children(system) {
        if child.text == "name-server" {
            found.extend(active_children(child).map(|server| name_server(&server.text, server)));
        } else if let Some(address) = child.arg("name-server") {
            found.push(name_server(address, child));
        }
    }
    found
}

fn name_server(address: &str, line: &ConfigLine) -> NameServer {
    NameServer {
        address: unquote(address).to_string(),
        routing_instance: arg_of(line, "routing-instance").map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_host_name() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert_eq!(system.host_name.as_deref(), Some("pe1"));
    }

    #[test]
    fn host_name_is_optional() {
        let system = system(&juniper::parse_braces("system {\n    domain-name example.net;\n}\n")).unwrap();
        assert_eq!(system.host_name, None);
    }

    #[test]
    fn skips_inactive_host_name() {
        let system = system(&juniper::parse_braces("system {\n    inactive: host-name pe1;\n}\n")).unwrap();
        assert_eq!(system.host_name, None);
    }

    #[test]
    fn extracts_name_server_block() {
        let system = system(&juniper::parse_braces(
            "system {\n    name-server {\n        192.0.2.53;\n        2001:db8::53;\n    }\n}\n",
        ))
        .unwrap();
        let addresses: Vec<_> = system.name_servers.iter().map(|s| s.address.as_str()).collect();
        assert_eq!(addresses, ["192.0.2.53", "2001:db8::53"]);
        assert_eq!(system.name_servers[0].routing_instance, None);
    }

    #[test]
    fn name_server_in_management_instance() {
        let system = system(&juniper::parse_braces(
            "system {\n    name-server {\n        192.0.2.53 {\n            routing-instance mgmt_junos;\n        }\n    }\n}\n",
        ))
        .unwrap();
        assert_eq!(system.name_servers[0].address, "192.0.2.53");
        assert_eq!(system.name_servers[0].routing_instance.as_deref(), Some("mgmt_junos"));
    }

    #[test]
    fn extracts_one_line_name_servers_and_skips_inactive() {
        let system = system(&juniper::parse_braces(
            "system {\n    name-server 192.0.2.53;\n    name-server 192.0.2.54;\n    inactive: name-server 192.0.2.55;\n}\n",
        ))
        .unwrap();
        let addresses: Vec<_> = system.name_servers.iter().map(|s| s.address.as_str()).collect();
        assert_eq!(addresses, ["192.0.2.53", "192.0.2.54"]);
    }

    #[test]
    fn no_system_is_none() {
        assert!(system(&juniper::parse_braces("interfaces {\n    lo0;\n}\n")).is_none());
    }
}
