//! `system { ... }`.

use crate::{
    active_children, arg_of, args_of, child_block, first_word, has_leaf, list_values, named_children, split_name,
    top_level, unquote,
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

/// A `<facility> <severity>` line selecting which messages go to a syslog destination.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SyslogRule {
    /// e.g. `"any"`, `"authorization"`, `"interactive-commands"`.
    pub facility: String,
    /// e.g. `"info"`, `"notice"`, `"any"`, `"none"`.
    pub severity: String,
}

/// A `host <address>` destination.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SyslogHost {
    /// Address, hostname, or `other-routing-engine`.
    pub address: String,
    pub rules: Vec<SyslogRule>,
    pub port: Option<String>,
    pub source_address: Option<String>,
    pub routing_instance: Option<String>,
    pub facility_override: Option<String>,
    pub log_prefix: Option<String>,
}

/// A `file <name>` or `user <name>` destination.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SyslogTarget {
    /// File name, or user name (`*` for all users).
    pub name: String,
    pub rules: Vec<SyslogRule>,
}

/// The `system syslog` configuration. Archive settings are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Syslog {
    pub hosts: Vec<SyslogHost>,
    pub files: Vec<SyslogTarget>,
    pub users: Vec<SyslogTarget>,
    pub console: Vec<SyslogRule>,
    pub source_address: Option<String>,
    /// e.g. `"year millisecond"`.
    pub time_format: Option<String>,
}

/// A login `class`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LoginClass {
    pub name: String,
    pub permissions: Vec<String>,
    /// Minutes.
    pub idle_timeout: Option<String>,
    /// The regular expressions as written, e.g. `"(show)|(ping)"`.
    pub allow_commands: Option<String>,
    pub deny_commands: Option<String>,
    pub allow_configuration: Option<String>,
    pub deny_configuration: Option<String>,
}

/// A login `user`. Passwords and keys are not extracted, only which authentication methods are present.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LoginUser {
    pub name: String,
    pub class: Option<String>,
    pub uid: Option<String>,
    pub full_name: Option<String>,
    /// One entry per authentication statement, e.g. `"encrypted-password"`, `"ssh-rsa"`.
    pub authentication_methods: Vec<String>,
}

/// The `system login` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Login {
    /// The banner shown before login.
    pub message: Option<String>,
    /// The text shown after login.
    pub announcement: Option<String>,
    pub classes: Vec<LoginClass>,
    pub users: Vec<LoginUser>,
}

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
}

/// The top-level `system` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct System {
    pub host_name: Option<String>,
    pub domain_name: Option<String>,
    pub name_servers: Vec<NameServer>,
    pub ntp: Option<Ntp>,
    pub syslog: Option<Syslog>,
    pub login: Option<Login>,
    pub services: Option<Services>,
}

/// The top-level `system` block, or `None` if there is none.
pub fn system(config: &Config) -> Option<System> {
    let system = top_level(config, "system")?;
    Some(System {
        host_name: arg_of(system, "host-name").map(str::to_string),
        domain_name: arg_of(system, "domain-name").map(str::to_string),
        name_servers: name_servers(system),
        ntp: child_block(system, "ntp").map(parse_ntp),
        syslog: child_block(system, "syslog").map(parse_syslog),
        login: child_block(system, "login").map(parse_login),
        services: child_block(system, "services").map(parse_services),
    })
}

fn parse_services(services: &ConfigLine) -> Services {
    let web = child_block(services, "web-management");
    let netconf = child_block(services, "netconf");
    Services {
        ssh: child_block(services, "ssh").map(parse_ssh),
        telnet: child_block(services, "telnet").is_some(),
        ftp: child_block(services, "ftp").is_some(),
        netconf_ssh: netconf.is_some_and(|n| child_block(n, "ssh").is_some()),
        http: web.is_some_and(|w| child_block(w, "http").is_some()),
        https: web.is_some_and(|w| child_block(w, "https").is_some()),
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

fn parse_login(login: &ConfigLine) -> Login {
    Login {
        message: arg_of(login, "message").map(str::to_string),
        announcement: arg_of(login, "announcement").map(str::to_string),
        classes: named_children(login, "class")
            .map(|(name, class)| LoginClass {
                name: name.to_string(),
                permissions: args_of(class, "permissions")
                    .into_iter()
                    .flat_map(list_values)
                    .collect(),
                idle_timeout: arg_of(class, "idle-timeout").map(str::to_string),
                allow_commands: arg_of(class, "allow-commands").map(str::to_string),
                deny_commands: arg_of(class, "deny-commands").map(str::to_string),
                allow_configuration: arg_of(class, "allow-configuration").map(str::to_string),
                deny_configuration: arg_of(class, "deny-configuration").map(str::to_string),
            })
            .collect(),
        users: named_children(login, "user")
            .map(|(name, user)| LoginUser {
                name: name.to_string(),
                class: arg_of(user, "class").map(str::to_string),
                uid: arg_of(user, "uid").map(str::to_string),
                full_name: arg_of(user, "full-name").map(str::to_string),
                authentication_methods: child_block(user, "authentication")
                    .map(|auth| {
                        // Secrets may carry a trailing `## SECRET-DATA` marker; only the method name is kept.
                        active_children(auth)
                            .filter_map(|c| c.text.split_whitespace().next())
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default(),
            })
            .collect(),
    }
}

fn parse_syslog(syslog: &ConfigLine) -> Syslog {
    Syslog {
        hosts: named_children(syslog, "host")
            .map(|(address, host)| SyslogHost {
                address: address.to_string(),
                rules: syslog_rules(host),
                port: arg_of(host, "port").map(str::to_string),
                source_address: arg_of(host, "source-address").map(str::to_string),
                routing_instance: arg_of(host, "routing-instance").map(str::to_string),
                facility_override: arg_of(host, "facility-override").map(str::to_string),
                log_prefix: arg_of(host, "log-prefix").map(str::to_string),
            })
            .collect(),
        files: syslog_targets(syslog, "file"),
        users: syslog_targets(syslog, "user"),
        console: child_block(syslog, "console").map(syslog_rules).unwrap_or_default(),
        source_address: arg_of(syslog, "source-address").map(str::to_string),
        time_format: arg_of(syslog, "time-format").map(str::to_string),
    }
}

fn syslog_targets(syslog: &ConfigLine, keyword: &str) -> Vec<SyslogTarget> {
    named_children(syslog, keyword)
        .map(|(name, target)| SyslogTarget {
            name: name.to_string(),
            rules: syslog_rules(target),
        })
        .collect()
}

/// Children written `<facility> <severity>`, which tells them apart from options such as `port 514`.
fn syslog_rules(destination: &ConfigLine) -> Vec<SyslogRule> {
    const SEVERITIES: [&str; 10] = [
        "any",
        "none",
        "emergency",
        "alert",
        "critical",
        "error",
        "warning",
        "notice",
        "info",
        "debug",
    ];
    active_children(destination)
        .filter_map(|c| {
            let (facility, severity) = c.text.split_once(' ')?;
            SEVERITIES.contains(&severity).then(|| SyslogRule {
                facility: facility.to_string(),
                severity: severity.to_string(),
            })
        })
        .collect()
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

    const SYSLOG: &str = r#"
system {
    syslog {
        user * {
            any emergency;
        }
        host 192.0.2.5 {
            any notice;
            authorization info;
            port 514;
            source-address 192.0.2.1;
            routing-instance mgmt_junos;
            facility-override local3;
            log-prefix pe1;
        }
        host other-routing-engine {
            any critical;
        }
        file messages {
            any notice;
            authorization info;
            explicit-priority;
        }
        file interactive-commands {
            interactive-commands any;
        }
        inactive: file debug {
            any debug;
        }
        console {
            any critical;
        }
        source-address 192.0.2.1;
        time-format year millisecond;
    }
}
"#;

    #[test]
    fn extracts_syslog_hosts() {
        let syslog = system(&juniper::parse_braces(SYSLOG)).unwrap().syslog.unwrap();
        let addresses: Vec<_> = syslog.hosts.iter().map(|h| h.address.as_str()).collect();
        assert_eq!(addresses, ["192.0.2.5", "other-routing-engine"]);

        let host = &syslog.hosts[0];
        assert_eq!(host.rules.len(), 2);
        assert_eq!(host.rules[1].facility, "authorization");
        assert_eq!(host.rules[1].severity, "info");
        assert_eq!(host.port.as_deref(), Some("514"));
        assert_eq!(host.source_address.as_deref(), Some("192.0.2.1"));
        assert_eq!(host.routing_instance.as_deref(), Some("mgmt_junos"));
        assert_eq!(host.facility_override.as_deref(), Some("local3"));
        assert_eq!(host.log_prefix.as_deref(), Some("pe1"));
    }

    #[test]
    fn extracts_syslog_files_users_and_console() {
        let syslog = system(&juniper::parse_braces(SYSLOG)).unwrap().syslog.unwrap();

        let files: Vec<_> = syslog.files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(files, ["messages", "interactive-commands"]);
        // `explicit-priority` is an option, not a facility/severity rule.
        assert_eq!(syslog.files[0].rules.len(), 2);
        assert_eq!(syslog.files[1].rules[0].facility, "interactive-commands");
        assert_eq!(syslog.files[1].rules[0].severity, "any");

        assert_eq!(syslog.users[0].name, "*");
        assert_eq!(syslog.users[0].rules[0].severity, "emergency");
        assert_eq!(syslog.console[0].facility, "any");
        assert_eq!(syslog.console[0].severity, "critical");
    }

    #[test]
    fn extracts_syslog_global_settings() {
        let syslog = system(&juniper::parse_braces(SYSLOG)).unwrap().syslog.unwrap();
        assert_eq!(syslog.source_address.as_deref(), Some("192.0.2.1"));
        assert_eq!(syslog.time_format.as_deref(), Some("year millisecond"));
    }

    #[test]
    fn no_syslog_is_none() {
        assert!(system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n"))
            .unwrap()
            .syslog
            .is_none());
    }

    const LOGIN: &str = r#"
system {
    login {
        message "Authorized access only";
        announcement "Maintenance window Sunday";
        class NOC {
            permissions [ view view-configuration ];
            idle-timeout 15;
            allow-commands "(ping)|(traceroute)";
            deny-configuration ".*";
        }
        class OPS {
            permissions all;
            deny-commands "(request system)";
            allow-configuration "(interfaces)|(protocols)";
        }
        user admin {
            uid 2000;
            class super-user;
            full-name "Network Admin";
            authentication {
                encrypted-password "$6$salt$hash"; ## SECRET-DATA
                ssh-ed25519 "ssh-ed25519 AAAAC3Nza admin@host"; ## SECRET-DATA
            }
        }
        user monitor {
            class NOC;
        }
    }
}
"#;

    #[test]
    fn extracts_login_banner_and_classes() {
        let login = system(&juniper::parse_braces(LOGIN)).unwrap().login.unwrap();
        assert_eq!(login.message.as_deref(), Some("Authorized access only"));
        assert_eq!(login.announcement.as_deref(), Some("Maintenance window Sunday"));

        assert_eq!(login.classes.len(), 2);
        let noc = &login.classes[0];
        assert_eq!(noc.name, "NOC");
        assert_eq!(noc.permissions, ["view", "view-configuration"]);
        assert_eq!(noc.idle_timeout.as_deref(), Some("15"));
        assert_eq!(noc.allow_commands.as_deref(), Some("(ping)|(traceroute)"));
        assert_eq!(noc.deny_configuration.as_deref(), Some(".*"));
        assert_eq!(noc.deny_commands, None);

        let ops = &login.classes[1];
        assert_eq!(ops.permissions, ["all"]);
        assert_eq!(ops.deny_commands.as_deref(), Some("(request system)"));
        assert_eq!(ops.allow_configuration.as_deref(), Some("(interfaces)|(protocols)"));
    }

    #[test]
    fn extracts_login_users_without_secrets() {
        let login = system(&juniper::parse_braces(LOGIN)).unwrap().login.unwrap();
        assert_eq!(login.users.len(), 2);

        let admin = &login.users[0];
        assert_eq!(admin.name, "admin");
        assert_eq!(admin.class.as_deref(), Some("super-user"));
        assert_eq!(admin.uid.as_deref(), Some("2000"));
        assert_eq!(admin.full_name.as_deref(), Some("Network Admin"));
        assert_eq!(admin.authentication_methods, ["encrypted-password", "ssh-ed25519"]);

        let monitor = &login.users[1];
        assert_eq!(monitor.class.as_deref(), Some("NOC"));
        assert!(monitor.authentication_methods.is_empty());
    }

    #[test]
    fn no_login_is_none() {
        assert!(system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n"))
            .unwrap()
            .login
            .is_none());
    }

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
    fn no_services_is_none() {
        assert!(system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n"))
            .unwrap()
            .services
            .is_none());
    }

    #[test]
    fn extracts_domain_name() {
        let system = system(&juniper::parse_braces(
            "system {\n    host-name pe1;\n    domain-name example.net;\n}\n",
        ))
        .unwrap();
        assert_eq!(system.domain_name.as_deref(), Some("example.net"));
    }

    #[test]
    fn domain_name_is_optional() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert_eq!(system.domain_name, None);
    }

    #[test]
    fn no_system_is_none() {
        assert!(system(&juniper::parse_braces("interfaces {\n    lo0;\n}\n")).is_none());
    }
}
