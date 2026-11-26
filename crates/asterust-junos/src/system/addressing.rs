//! Name resolution and addressing settings under `system`.

use crate::{active_children, arg_of, args_of, child_block, list_values, split_name, unquote};
use asterust::config::ConfigLine;

/// A DNS server from `name-server`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct NameServer {
    pub address: String,
    /// Set when the server is reached through a routing instance such as `mgmt_junos`.
    pub routing_instance: Option<String>,
}

/// A `static-host-mapping <hostname>` entry.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct StaticHostMapping {
    pub hostname: String,
    pub aliases: Vec<String>,
    pub inet: Vec<String>,
    pub inet6: Vec<String>,
    /// The ISO system identifier, e.g. `0100.2550.0001`.
    pub sysid: Option<String>,
}

/// A `backup-router` or `inet6-backup-router`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BackupRouter {
    pub address: String,
    /// Empty when the router is reachable for all destinations.
    pub destinations: Vec<String>,
}

pub(super) fn static_host_mappings(system: &ConfigLine) -> Vec<StaticHostMapping> {
    child_block(system, "static-host-mapping")
        .map(|mappings| active_children(mappings).map(parse_static_host_mapping).collect())
        .unwrap_or_default()
}

fn parse_static_host_mapping(mapping: &ConfigLine) -> StaticHostMapping {
    let list = |keyword: &str| args_of(mapping, keyword).into_iter().flat_map(list_values).collect();
    StaticHostMapping {
        hostname: unquote(&mapping.text).to_string(),
        aliases: list("alias"),
        inet: list("inet"),
        inet6: list("inet6"),
        sysid: arg_of(mapping, "sysid").map(str::to_string),
    }
}

/// `<address> [destination <prefix | [ prefixes ]>]`.
pub(super) fn parse_backup_router(arg: &str) -> BackupRouter {
    let (address, rest) = split_name(arg);
    BackupRouter {
        address: address.to_string(),
        destinations: rest.strip_prefix("destination").map(list_values).unwrap_or_default(),
    }
}

/// Servers written as a `name-server { <address>; ... }` block, or one per `name-server <address>` line.
pub(super) fn name_servers(system: &ConfigLine) -> Vec<NameServer> {
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
    use crate::system::system;
    use asterust::config::juniper;

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
    fn extracts_domain_search_list() {
        let system = system(&juniper::parse_braces(
            "system {\n    domain-search [ example.net lab.example.net ];\n}\n",
        ))
        .unwrap();
        assert_eq!(system.domain_search, ["example.net", "lab.example.net"]);
    }

    #[test]
    fn extracts_static_host_mappings() {
        let system = system(&juniper::parse_braces(
            r#"
system {
    static-host-mapping {
        p1.example.net {
            inet [ 10.255.0.2 10.255.1.2 ];
            inet6 2001:db8::2;
            alias [ p1 core1 ];
            sysid 0100.2550.0002;
        }
        p2.example.net {
            inet 10.255.0.3;
        }
    }
}
"#,
        ))
        .unwrap();
        assert_eq!(system.static_host_mappings.len(), 2);

        let p1 = &system.static_host_mappings[0];
        assert_eq!(p1.hostname, "p1.example.net");
        assert_eq!(p1.inet, ["10.255.0.2", "10.255.1.2"]);
        assert_eq!(p1.inet6, ["2001:db8::2"]);
        assert_eq!(p1.aliases, ["p1", "core1"]);
        assert_eq!(p1.sysid.as_deref(), Some("0100.2550.0002"));

        let p2 = &system.static_host_mappings[1];
        assert_eq!(p2.inet, ["10.255.0.3"]);
        assert!(p2.aliases.is_empty());
        assert_eq!(p2.sysid, None);
    }

    #[test]
    fn extracts_backup_routers() {
        let system = system(&juniper::parse_braces(
            "system {\n    backup-router 192.0.2.1 destination [ 10.0.0.0/8 172.16.0.0/12 ];\n    inet6-backup-router 2001:db8::1;\n}\n",
        ))
        .unwrap();
        let backup = system.backup_router.unwrap();
        assert_eq!(backup.address, "192.0.2.1");
        assert_eq!(backup.destinations, ["10.0.0.0/8", "172.16.0.0/12"]);

        let backup6 = system.inet6_backup_router.unwrap();
        assert_eq!(backup6.address, "2001:db8::1");
        assert!(backup6.destinations.is_empty());
    }

    #[test]
    fn single_destination_backup_router() {
        let system = system(&juniper::parse_braces(
            "system {\n    backup-router 192.0.2.1 destination 10.0.0.0/8;\n}\n",
        ))
        .unwrap();
        assert_eq!(system.backup_router.unwrap().destinations, ["10.0.0.0/8"]);
    }

    #[test]
    fn addressing_settings_are_optional() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.domain_search.is_empty());
        assert!(system.static_host_mappings.is_empty());
        assert!(system.backup_router.is_none());
        assert!(system.inet6_backup_router.is_none());
    }
}
