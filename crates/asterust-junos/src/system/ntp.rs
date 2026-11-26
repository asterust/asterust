//! `ntp`
//! `system { ntp { ... } }`.
//!
//! # See also
//!
//! - [ntp](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/ntp-edit-system.html)

use crate::{active_children, arg_of, args_of, first_word, has_leaf, list_values, split_name};
use asterust::config::ConfigLine;

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

pub(super) fn parse_ntp(ntp: &ConfigLine) -> Ntp {
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

/// A host written `<address> [key n] [prefer] [version n] [routing-instance x]`, with the options
/// on the line or in a block.
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

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

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
}
