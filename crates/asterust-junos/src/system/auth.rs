//! Authentication settings under `system`: root, RADIUS and TACACS+.

use crate::{active_children, arg_of, child_block, has_leaf, split_name, unquote};
use asterust::config::ConfigLine;

/// The `system root-authentication` methods. Passwords and keys are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct RootAuthentication {
    /// One entry per statement, e.g. `"encrypted-password"`, `"ssh-rsa"`, `"no-public-keys"`.
    pub methods: Vec<String>,
}

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

/// Servers written as `<keyword> <address> { ... }`, or as `<keyword> { <address> { ... } }`, with their option lines.
fn servers<'a>(system: &'a ConfigLine, keyword: &str) -> Vec<(&'a str, &'a ConfigLine)> {
    let mut found = Vec::new();
    for child in active_children(system) {
        if child.text == keyword {
            found.extend(active_children(child).map(|server| (unquote(&server.text), server)));
        } else if let Some(arg) = child.arg(keyword) {
            found.push((split_name(arg).0, child));
        }
    }
    found
}

pub(super) fn radius_servers(system: &ConfigLine) -> Vec<RadiusServer> {
    servers(system, "radius-server")
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

pub(super) fn tacplus_servers(system: &ConfigLine) -> Vec<TacplusServer> {
    servers(system, "tacplus-server")
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

pub(super) fn root_authentication(system: &ConfigLine) -> Option<RootAuthentication> {
    child_block(system, "root-authentication").map(|auth| RootAuthentication {
        methods: authentication_methods(auth),
    })
}

/// The name of each statement in an `authentication` block. Secrets can carry a trailing `## SECRET-DATA` marker,
/// so only the first word is kept.
pub(super) fn authentication_methods(auth: &ConfigLine) -> Vec<String> {
    active_children(auth)
        .filter_map(|c| c.text.split_whitespace().next())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    #[test]
    fn extracts_root_authentication_methods_without_secrets() {
        let system = system(&juniper::parse_braces(
            r#"
system {
    root-authentication {
        encrypted-password "$6$salt$hash"; ## SECRET-DATA
        ssh-rsa "ssh-rsa AAAAB3Nza root@host"; ## SECRET-DATA
        ssh-rsa "ssh-rsa AAAAB3Nzb ops@host" {
            from 192.0.2.0/24;
        }
    }
}
"#,
        ))
        .unwrap();
        let root = system.root_authentication.unwrap();
        assert_eq!(root.methods, ["encrypted-password", "ssh-rsa", "ssh-rsa"]);
    }

    #[test]
    fn root_authentication_can_disable_public_keys() {
        let system = system(&juniper::parse_braces(
            "system {\n    root-authentication {\n        no-public-keys;\n    }\n}\n",
        ))
        .unwrap();
        assert_eq!(system.root_authentication.unwrap().methods, ["no-public-keys"]);
    }

    #[test]
    fn no_root_authentication_is_none() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.root_authentication.is_none());
    }

    #[test]
    fn extracts_authentication_order() {
        let system = system(&juniper::parse_braces(
            "system {\n    authentication-order [ tacplus radius password ];\n}\n",
        ))
        .unwrap();
        assert_eq!(system.authentication_order, ["tacplus", "radius", "password"]);
    }

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

    #[test]
    fn authentication_settings_are_optional() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.authentication_order.is_empty());
        assert!(system.radius_servers.is_empty());
        assert!(system.tacplus_servers.is_empty());
    }
}
