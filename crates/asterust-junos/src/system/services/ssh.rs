//! `ssh`
//! `system { services { ssh { ... } } }`.
//!
//! # See also
//!
//! - [ssh (System Services)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/ssh-edit-system.html)

use crate::{arg_of, args_of, has_leaf, list_values};
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

pub(super) fn parse_ssh(ssh: &ConfigLine) -> SshService {
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
    use crate::system::system;
    use asterust::config::juniper;

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
}
