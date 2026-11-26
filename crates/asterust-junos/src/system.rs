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

/// A syslog `archive` statement. `archive-sites`, `start-time` and `transfer-interval` are not extracted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct SyslogArchive {
    /// Maximum number of archived files.
    pub files: Option<String>,
    /// Maximum file size, e.g. `"1m"`.
    pub size: Option<String>,
    /// `Some(true)` for `world-readable`, `Some(false)` for `no-world-readable`.
    pub world_readable: Option<bool>,
}

/// A `file <name>` or `user <name>` destination.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SyslogTarget {
    /// File name, or user name (`*` for all users).
    pub name: String,
    pub rules: Vec<SyslogRule>,
    /// Only files have an archive.
    pub archive: Option<SyslogArchive>,
}

/// The `system syslog` configuration.
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
    /// The default archive settings that files inherit.
    pub archive: Option<SyslogArchive>,
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
    pub outbound_ssh: Vec<OutboundSshClient>,
}

/// The `system root-authentication` methods. Passwords and keys are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct RootAuthentication {
    /// One entry per statement, e.g. `"encrypted-password"`, `"ssh-rsa"`, `"no-public-keys"`.
    pub methods: Vec<String>,
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

/// A `system location` block. The Bellcore coordinates (`hcoord`, `vcoord`, `lata`, `npa-nxx`) are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Location {
    pub building: Option<String>,
    pub floor: Option<String>,
    pub rack: Option<String>,
    pub country_code: Option<String>,
    pub postal_code: Option<String>,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    /// Feet above sea level.
    pub altitude: Option<String>,
}

/// A `system ports console` or `auxiliary` block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SerialPort {
    pub disable: bool,
    pub insecure: bool,
    pub log_out_on_disconnect: bool,
    /// From `type`, e.g. `vt100`.
    pub terminal_type: Option<String>,
    /// `mini-usb` or `rj45`; only on the auxiliary port.
    pub port_type: Option<String>,
}

/// The `system ports` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Ports {
    pub console: Option<SerialPort>,
    pub auxiliary: Option<SerialPort>,
}

/// The top-level `system` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct System {
    pub host_name: Option<String>,
    pub domain_name: Option<String>,
    /// e.g. `"UTC"` or `"America/New_York"`.
    pub time_zone: Option<String>,
    pub domain_search: Vec<String>,
    pub static_host_mappings: Vec<StaticHostMapping>,
    pub backup_router: Option<BackupRouter>,
    pub inet6_backup_router: Option<BackupRouter>,
    /// Methods in the order tried: `password`, `radius`, `tacplus`.
    pub authentication_order: Vec<String>,
    pub radius_servers: Vec<RadiusServer>,
    pub tacplus_servers: Vec<TacplusServer>,
    pub location: Option<Location>,
    pub ports: Option<Ports>,
    pub max_configurations_on_flash: Option<String>,
    pub max_configuration_rollbacks: Option<String>,
    pub no_redirects: bool,
    pub root_authentication: Option<RootAuthentication>,
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
        time_zone: arg_of(system, "time-zone").map(str::to_string),
        domain_search: args_of(system, "domain-search")
            .into_iter()
            .flat_map(list_values)
            .collect(),
        static_host_mappings: child_block(system, "static-host-mapping")
            .map(|mappings| active_children(mappings).map(parse_static_host_mapping).collect())
            .unwrap_or_default(),
        backup_router: arg_of(system, "backup-router").map(parse_backup_router),
        inet6_backup_router: arg_of(system, "inet6-backup-router").map(parse_backup_router),
        authentication_order: args_of(system, "authentication-order")
            .into_iter()
            .flat_map(list_values)
            .collect(),
        radius_servers: servers(system, "radius-server")
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
            .collect(),
        tacplus_servers: servers(system, "tacplus-server")
            .into_iter()
            .map(|(address, server)| TacplusServer {
                address: address.to_string(),
                port: arg_of(server, "port").map(str::to_string),
                timeout: arg_of(server, "timeout").map(str::to_string),
                source_address: arg_of(server, "source-address").map(str::to_string),
                routing_instance: arg_of(server, "routing-instance").map(str::to_string),
                single_connection: has_leaf(server, "single-connection"),
            })
            .collect(),
        location: child_block(system, "location").map(parse_location),
        ports: child_block(system, "ports").map(|ports| Ports {
            console: child_block(ports, "console").map(parse_serial_port),
            auxiliary: child_block(ports, "auxiliary").map(parse_serial_port),
        }),
        max_configurations_on_flash: arg_of(system, "max-configurations-on-flash").map(str::to_string),
        max_configuration_rollbacks: arg_of(system, "max-configuration-rollbacks").map(str::to_string),
        no_redirects: has_leaf(system, "no-redirects"),
        root_authentication: child_block(system, "root-authentication").map(|auth| RootAuthentication {
            methods: authentication_methods(auth),
        }),
        name_servers: name_servers(system),
        ntp: child_block(system, "ntp").map(parse_ntp),
        syslog: child_block(system, "syslog").map(parse_syslog),
        login: child_block(system, "login").map(parse_login),
        services: child_block(system, "services").map(parse_services),
    })
}

fn parse_location(location: &ConfigLine) -> Location {
    let field = |keyword: &str| arg_of(location, keyword).map(str::to_string);
    Location {
        building: field("building"),
        floor: field("floor"),
        rack: field("rack"),
        country_code: field("country-code"),
        postal_code: field("postal-code"),
        latitude: field("latitude"),
        longitude: field("longitude"),
        altitude: field("altitude"),
    }
}

fn parse_serial_port(port: &ConfigLine) -> SerialPort {
    SerialPort {
        disable: has_leaf(port, "disable"),
        insecure: has_leaf(port, "insecure"),
        log_out_on_disconnect: has_leaf(port, "log-out-on-disconnect"),
        terminal_type: arg_of(port, "type").map(str::to_string),
        port_type: arg_of(port, "port-type").map(str::to_string),
    }
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
fn parse_backup_router(arg: &str) -> BackupRouter {
    let (address, rest) = split_name(arg);
    BackupRouter {
        address: address.to_string(),
        destinations: rest.strip_prefix("destination").map(list_values).unwrap_or_default(),
    }
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
        outbound_ssh: child_block(services, "outbound-ssh")
            .map(|outbound| {
                named_children(outbound, "client")
                    .map(|(id, client)| parse_outbound_client(id, client))
                    .collect()
            })
            .unwrap_or_default(),
    }
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

/// The name of each statement in an `authentication` block. Secrets can carry a trailing `## SECRET-DATA` marker,
/// so only the first word is kept.
fn authentication_methods(auth: &ConfigLine) -> Vec<String> {
    active_children(auth)
        .filter_map(|c| c.text.split_whitespace().next())
        .map(str::to_string)
        .collect()
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
                    .map(authentication_methods)
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
        archive: syslog_archive(syslog),
    }
}

fn syslog_targets(syslog: &ConfigLine, keyword: &str) -> Vec<SyslogTarget> {
    named_children(syslog, keyword)
        .map(|(name, target)| SyslogTarget {
            name: name.to_string(),
            rules: syslog_rules(target),
            archive: syslog_archive(target),
        })
        .collect()
}

/// An `archive` written as a block or on one line (`archive size 1m files 5 world-readable;`).
fn syslog_archive(parent: &ConfigLine) -> Option<SyslogArchive> {
    let archive = active_children(parent).find(|c| c.text == "archive" || c.arg("archive").is_some())?;
    // Both forms reduce to one stream of words: the options on the line itself, then one statement per child.
    let mut words = archive
        .text
        .split_whitespace()
        .skip(1)
        .chain(active_children(archive).flat_map(|c| c.text.split_whitespace()));
    let mut settings = SyslogArchive::default();
    while let Some(word) = words.next() {
        match word {
            "files" => settings.files = words.next().map(str::to_string),
            "size" => settings.size = words.next().map(str::to_string),
            "world-readable" => settings.world_readable = Some(true),
            "no-world-readable" => settings.world_readable = Some(false),
            _ => {}
        }
    }
    Some(settings)
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
    fn extracts_syslog_archive_in_line_and_block_form() {
        let syslog = system(&juniper::parse_braces(
            r#"
system {
    syslog {
        archive {
            files 20;
            size 5m;
            world-readable;
        }
        file messages {
            any notice;
            archive size 100k files 10 no-world-readable;
        }
        file plain {
            any info;
        }
    }
}
"#,
        ))
        .unwrap()
        .syslog
        .unwrap();

        let default = syslog.archive.unwrap();
        assert_eq!(default.files.as_deref(), Some("20"));
        assert_eq!(default.size.as_deref(), Some("5m"));
        assert_eq!(default.world_readable, Some(true));

        let messages = syslog.files[0].archive.as_ref().unwrap();
        assert_eq!(messages.files.as_deref(), Some("10"));
        assert_eq!(messages.size.as_deref(), Some("100k"));
        assert_eq!(messages.world_readable, Some(false));

        assert_eq!(syslog.files[1].archive, None);
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
    fn extracts_time_zone() {
        let system = system(&juniper::parse_braces("system {\n    time-zone America/New_York;\n}\n")).unwrap();
        assert_eq!(system.time_zone.as_deref(), Some("America/New_York"));
    }

    #[test]
    fn time_zone_is_optional() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert_eq!(system.time_zone, None);
    }

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

    #[test]
    fn extracts_location() {
        let system = system(&juniper::parse_braces(
            r#"
system {
    location {
        building "DC East";
        floor 2;
        rack 14;
        country-code US;
        postal-code 10001;
        latitude 40.7128;
        longitude -74.006;
        altitude 33;
    }
}
"#,
        ))
        .unwrap();
        let location = system.location.unwrap();
        assert_eq!(location.building.as_deref(), Some("DC East"));
        assert_eq!(location.floor.as_deref(), Some("2"));
        assert_eq!(location.rack.as_deref(), Some("14"));
        assert_eq!(location.country_code.as_deref(), Some("US"));
        assert_eq!(location.postal_code.as_deref(), Some("10001"));
        assert_eq!(location.latitude.as_deref(), Some("40.7128"));
        assert_eq!(location.longitude.as_deref(), Some("-74.006"));
        assert_eq!(location.altitude.as_deref(), Some("33"));
    }

    #[test]
    fn extracts_console_and_auxiliary_ports() {
        let system = system(&juniper::parse_braces(
            r#"
system {
    ports {
        console {
            log-out-on-disconnect;
            type vt100;
            insecure;
        }
        auxiliary {
            disable;
            port-type rj45;
        }
    }
}
"#,
        ))
        .unwrap();
        let ports = system.ports.unwrap();

        let console = ports.console.unwrap();
        assert!(console.log_out_on_disconnect);
        assert!(console.insecure);
        assert!(!console.disable);
        assert_eq!(console.terminal_type.as_deref(), Some("vt100"));
        assert_eq!(console.port_type, None);

        let auxiliary = ports.auxiliary.unwrap();
        assert!(auxiliary.disable);
        assert_eq!(auxiliary.port_type.as_deref(), Some("rj45"));
    }

    #[test]
    fn extracts_configuration_limits_and_no_redirects() {
        let system = system(&juniper::parse_braces(
            "system {\n    max-configurations-on-flash 5;\n    max-configuration-rollbacks 20;\n    no-redirects;\n}\n",
        ))
        .unwrap();
        assert_eq!(system.max_configurations_on_flash.as_deref(), Some("5"));
        assert_eq!(system.max_configuration_rollbacks.as_deref(), Some("20"));
        assert!(system.no_redirects);
    }

    #[test]
    fn device_settings_are_optional() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.location.is_none());
        assert!(system.ports.is_none());
        assert_eq!(system.max_configurations_on_flash, None);
        assert_eq!(system.max_configuration_rollbacks, None);
        assert!(!system.no_redirects);
    }

    #[test]
    fn no_system_is_none() {
        assert!(system(&juniper::parse_braces("interfaces {\n    lo0;\n}\n")).is_none());
    }
}
