//! `system { ... }`.

use crate::{
    active_children, arg_of, args_of, child_block, first_word, has_leaf, list_values, split_name, top_level, unquote,
};
use asterust::config::{Config, ConfigLine};

/// A DNS server from `name-server`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct NameServer {
    pub address: String,
    /// Set when the server is reached through a routing instance such as `mgmt_junos`.
    pub routing_instance: Option<String>,
}

/// A `server` or `peer` entry under `ntp`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct NtpHost {
    /// Address or hostname.
    pub address: String,
    /// Authentication key number.
    pub key: Option<String>,
    pub prefer: bool,
    pub version: Option<String>,
    pub routing_instance: Option<String>,
}

/// An `authentication-key`. The key value is not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct NtpAuthenticationKey {
    pub number: String,
    /// `md5`, `sha1` or `sha256`.
    pub key_type: Option<String>,
}

/// The `system ntp` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Ntp {
    pub boot_server: Option<String>,
    pub servers: Vec<NtpHost>,
    pub peers: Vec<NtpHost>,
    pub source_address: Option<String>,
    pub trusted_keys: Vec<String>,
    pub authentication_keys: Vec<NtpAuthenticationKey>,
    pub broadcast_client: bool,
}

/// The top-level `system` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct System {
    pub host_name: Option<String>,
    pub name_servers: Vec<NameServer>,
    pub ntp: Option<Ntp>,
}

/// The top-level `system` block, or `None` if there is none.
pub fn system(config: &Config) -> Option<System> {
    let system = top_level(config, "system")?;
    Some(System {
        host_name: arg_of(system, "host-name").map(str::to_string),
        name_servers: name_servers(system),
        ntp: child_block(system, "ntp").map(parse_ntp),
    })
}

fn parse_ntp(ntp: &ConfigLine) -> Ntp {
    let hosts = |keyword: &str| -> Vec<NtpHost> {
        active_children(ntp)
            .filter_map(|c| Some(ntp_host(c.arg(keyword)?, c)))
            .collect()
    };
    Ntp {
        boot_server: arg_of(ntp, "boot-server").map(str::to_string),
        servers: hosts("server"),
        peers: hosts("peer"),
        source_address: arg_of(ntp, "source-address").and_then(first_word).map(str::to_string),
        trusted_keys: args_of(ntp, "trusted-key").into_iter().flat_map(list_values).collect(),
        authentication_keys: args_of(ntp, "authentication-key")
            .into_iter()
            .filter_map(ntp_authentication_key)
            .collect(),
        broadcast_client: has_leaf(ntp, "broadcast-client"),
    }
}

/// A host written `<address> [key n] [prefer] [version n] [routing-instance x]`, with the options on the line
/// or in a block.
fn ntp_host(arg: &str, line: &ConfigLine) -> NtpHost {
    let (address, options) = split_name(arg);
    let mut words = options.split_whitespace();
    let mut host = NtpHost {
        address: address.to_string(),
        key: arg_of(line, "key").map(str::to_string),
        prefer: has_leaf(line, "prefer"),
        version: arg_of(line, "version").map(str::to_string),
        routing_instance: arg_of(line, "routing-instance").map(str::to_string),
    };
    while let Some(word) = words.next() {
        match word {
            "prefer" => host.prefer = true,
            "key" => host.key = words.next().map(str::to_string),
            "version" => host.version = words.next().map(str::to_string),
            "routing-instance" => host.routing_instance = words.next().map(str::to_string),
            _ => {}
        }
    }
    host
}

/// `<number> type <type> value <secret>`; the secret is dropped.
fn ntp_authentication_key(arg: &str) -> Option<NtpAuthenticationKey> {
    let mut words = arg.split_whitespace();
    let number = words.next()?.to_string();
    let key_type = words.skip_while(|w| *w != "type").nth(1).map(str::to_string);
    Some(NtpAuthenticationKey { number, key_type })
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

    const NTP: &str = r#"
system {
    ntp {
        boot-server 192.0.2.10;
        source-address 192.0.2.1;
        authentication-key 1 type md5 value "$9$secret";
        authentication-key 2 type sha256 value "$9$other";
        trusted-key [ 1 2 ];
        server 192.0.2.10 key 1 prefer version 4;
        server 192.0.2.11;
        server ntp.example.net {
            key 2;
            prefer;
            version 3;
            routing-instance mgmt_junos;
        }
        inactive: server 192.0.2.12;
        peer 192.0.2.20 key 1;
        broadcast-client;
    }
}
"#;

    #[test]
    fn extracts_ntp_servers_in_line_and_block_form() {
        let ntp = system(&juniper::parse_braces(NTP)).unwrap().ntp.unwrap();
        let addresses: Vec<_> = ntp.servers.iter().map(|s| s.address.as_str()).collect();
        assert_eq!(addresses, ["192.0.2.10", "192.0.2.11", "ntp.example.net"]);

        let line = &ntp.servers[0];
        assert_eq!(line.key.as_deref(), Some("1"));
        assert!(line.prefer);
        assert_eq!(line.version.as_deref(), Some("4"));

        assert_eq!(ntp.servers[1].key, None);
        assert!(!ntp.servers[1].prefer);

        let block = &ntp.servers[2];
        assert_eq!(block.key.as_deref(), Some("2"));
        assert!(block.prefer);
        assert_eq!(block.version.as_deref(), Some("3"));
        assert_eq!(block.routing_instance.as_deref(), Some("mgmt_junos"));
    }

    #[test]
    fn extracts_ntp_peers_keys_and_global_settings() {
        let ntp = system(&juniper::parse_braces(NTP)).unwrap().ntp.unwrap();
        assert_eq!(ntp.peers.len(), 1);
        assert_eq!(ntp.peers[0].address, "192.0.2.20");
        assert_eq!(ntp.peers[0].key.as_deref(), Some("1"));
        assert_eq!(ntp.boot_server.as_deref(), Some("192.0.2.10"));
        assert_eq!(ntp.source_address.as_deref(), Some("192.0.2.1"));
        assert_eq!(ntp.trusted_keys, ["1", "2"]);
        assert!(ntp.broadcast_client);
    }

    #[test]
    fn ntp_authentication_keys_omit_the_secret() {
        let ntp = system(&juniper::parse_braces(NTP)).unwrap().ntp.unwrap();
        assert_eq!(ntp.authentication_keys.len(), 2);
        assert_eq!(ntp.authentication_keys[0].number, "1");
        assert_eq!(ntp.authentication_keys[0].key_type.as_deref(), Some("md5"));
        assert_eq!(ntp.authentication_keys[1].key_type.as_deref(), Some("sha256"));
    }

    #[test]
    fn no_ntp_is_none() {
        assert!(system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n"))
            .unwrap()
            .ntp
            .is_none());
    }

    #[test]
    fn no_system_is_none() {
        assert!(system(&juniper::parse_braces("interfaces {\n    lo0;\n}\n")).is_none());
    }
}
