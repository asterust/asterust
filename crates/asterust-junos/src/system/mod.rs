//! `system { ... }`.

mod accounting;
mod archival;
mod autoinstallation;
mod backup_router;
mod commit;
mod dynamic_profile_options;
mod extensions;
mod internet_options;
mod license;
mod location;
mod login;
mod master_password;
mod name_server;
mod ntp;
mod pixel_tracing;
mod ports;
mod processes;
mod radius_server;
mod root_authentication;
mod schema;
mod scripts;
mod services;
mod static_host_mapping;
mod syslog;
mod tacplus_server;

pub use accounting::{Accounting, AccountingDestination};
pub use archival::{Archival, ArchivalConfiguration};
pub use autoinstallation::{Autoinstallation, AutoinstallationInterface};
pub use backup_router::BackupRouter;
pub use commit::Commit;
pub use dynamic_profile_options::DynamicProfileOptions;
pub use extensions::{ExtensionApplication, ExtensionProvider, Extensions, ProviderLicense};
pub use internet_options::{IcmpRateLimit, InternetOptions};
pub use license::{License, LicenseAutoupdate, LicenseRenew};
pub use location::Location;
pub use login::{Login, LoginClass, LoginPassword, LoginRetryOptions, LoginUser};
pub use master_password::MasterPassword;
pub use name_server::NameServer;
pub use ntp::{Ntp, NtpAuthenticationKey, NtpHost};
pub use pixel_tracing::PixelTracing;
pub use ports::{Auxiliary, Console, Ports};
pub use processes::Process;
pub use radius_server::RadiusServer;
pub use root_authentication::RootAuthentication;
pub use schema::Schema;
pub use scripts::Scripts;
pub use services::{
    AaclStatisticsProfile, DhcpLocalServer, DhcpLocalServerGroup, FlowTapDtcp, FlowTapDtcpSsh,
    LocalPolicyDecisionFunction, Netconf, NetconfSsh, OutboundSshClient, OutboundSshServer, RedundancyInterface,
    ServiceDeployment, ServiceDeploymentServer, Services, SshService, StaticSubscriberGroup, StaticSubscriberProfile,
    StaticSubscribers, SubscriberManagement, SubscriberRedundancy, WebManagement, WebManagementHttp,
    WebManagementHttps, XnmClearText, XnmSsl,
};
pub use static_host_mapping::StaticHostMapping;
pub use syslog::{Syslog, SyslogArchive, SyslogHost, SyslogRule, SyslogStructuredData, SyslogTarget};
pub use tacplus_server::TacplusServer;

use crate::{active_children, arg_of, args_of, child_block, has_leaf, list_values, split_name, top_level, unquote};
use accounting::parse_accounting;
use archival::parse_archival;
use asterust::config::{Config, ConfigLine};
use autoinstallation::parse_autoinstallation;
use backup_router::parse_backup_router;
use commit::parse_commit;
use dynamic_profile_options::parse_dynamic_profile_options;
use extensions::parse_extensions;
use internet_options::parse_internet_options;
use license::parse_license;
use location::parse_location;
use login::parse_login;
use master_password::parse_master_password;
use name_server::name_servers;
use ntp::parse_ntp;
use pixel_tracing::parse_pixel_tracing;
use ports::parse_ports;
use processes::parse_processes;
use radius_server::radius_servers;
use root_authentication::root_authentication;
use schema::parse_schema;
use scripts::parse_scripts;
use services::parse_services;
use static_host_mapping::static_host_mappings;
use syslog::parse_syslog;
use tacplus_server::tacplus_servers;

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
    /// `default-address-selection`: use the loopback as the source of locally generated packets.
    pub default_address_selection: bool,
    pub internet_options: Option<InternetOptions>,
    pub commit: Option<Commit>,
    pub processes: Vec<Process>,
    pub dynamic_profile_options: Option<DynamicProfileOptions>,
    pub extensions: Option<Extensions>,
    pub autoinstallation: Option<Autoinstallation>,
    pub schema: Option<Schema>,
    pub pixel_tracing: Option<PixelTracing>,
    pub scripts: Option<Scripts>,
    pub no_multicast_echo: bool,
    pub no_ping_record_route: bool,
    pub no_ping_time_stamp: bool,
    pub auto_snapshot: bool,
    pub compress_configuration_files: bool,
    pub mirror_flash_on_disk: bool,
    pub switchover_on_routing_crash: bool,
    pub use_imported_time_zones: bool,
    pub archival: Option<Archival>,
    pub accounting: Option<Accounting>,
    pub root_authentication: Option<RootAuthentication>,
    pub master_password: Option<MasterPassword>,
    /// Method name of `diag-port-authentication`, e.g. `encrypted-password`; the password is not extracted.
    pub diag_port_authentication: Vec<String>,
    /// Method names of `pic-console-authentication`; the password is not extracted.
    pub pic_console_authentication: Vec<String>,
    pub license: Option<License>,
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
        static_host_mappings: static_host_mappings(system),
        backup_router: arg_of(system, "backup-router").map(parse_backup_router),
        inet6_backup_router: arg_of(system, "inet6-backup-router").map(parse_backup_router),
        authentication_order: args_of(system, "authentication-order")
            .into_iter()
            .flat_map(list_values)
            .collect(),
        radius_servers: radius_servers(system, "radius-server"),
        tacplus_servers: tacplus_servers(system, "tacplus-server"),
        location: child_block(system, "location").map(parse_location),
        ports: child_block(system, "ports").map(parse_ports),
        max_configurations_on_flash: arg_of(system, "max-configurations-on-flash").map(str::to_string),
        max_configuration_rollbacks: arg_of(system, "max-configuration-rollbacks").map(str::to_string),
        no_redirects: has_leaf(system, "no-redirects"),
        default_address_selection: has_leaf(system, "default-address-selection"),
        internet_options: child_block(system, "internet-options").map(parse_internet_options),
        commit: child_block(system, "commit").map(parse_commit),
        processes: child_block(system, "processes")
            .map(parse_processes)
            .unwrap_or_default(),
        dynamic_profile_options: child_block(system, "dynamic-profile-options").map(parse_dynamic_profile_options),
        extensions: child_block(system, "extensions").map(parse_extensions),
        autoinstallation: child_block(system, "autoinstallation").map(parse_autoinstallation),
        schema: child_block(system, "schema").map(parse_schema),
        pixel_tracing: child_block(system, "pixel-tracing").map(parse_pixel_tracing),
        scripts: child_block(system, "scripts").map(parse_scripts),
        no_multicast_echo: has_leaf(system, "no-multicast-echo"),
        no_ping_record_route: has_leaf(system, "no-ping-record-route"),
        no_ping_time_stamp: has_leaf(system, "no-ping-time-stamp"),
        auto_snapshot: has_leaf(system, "auto-snapshot"),
        compress_configuration_files: has_leaf(system, "compress-configuration-files"),
        mirror_flash_on_disk: has_leaf(system, "mirror-flash-on-disk"),
        switchover_on_routing_crash: has_leaf(system, "switchover-on-routing-crash"),
        use_imported_time_zones: has_leaf(system, "use-imported-time-zones"),
        archival: child_block(system, "archival").map(parse_archival),
        accounting: child_block(system, "accounting").map(parse_accounting),
        root_authentication: root_authentication(system),
        master_password: child_block(system, "master-password").map(parse_master_password),
        diag_port_authentication: password_methods(system, "diag-port-authentication"),
        pic_console_authentication: password_methods(system, "pic-console-authentication"),
        license: child_block(system, "license").map(parse_license),
        name_servers: name_servers(system),
        ntp: child_block(system, "ntp").map(parse_ntp),
        syslog: child_block(system, "syslog").map(parse_syslog),
        login: child_block(system, "login").map(parse_login),
        services: child_block(system, "services").map(parse_services),
    })
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

/// The method names of a password setting such as `diag-port-authentication`, written either as
/// `<keyword> encrypted-password "...";` or as a block. The passwords are dropped.
fn password_methods(system: &ConfigLine, keyword: &str) -> Vec<String> {
    let mut methods = Vec::new();
    for child in active_children(system) {
        if child.text == keyword {
            methods.extend(authentication_methods(child));
        } else if let Some(method) = child.arg(keyword).and_then(|arg| arg.split_whitespace().next()) {
            methods.push(method.to_string());
        }
    }
    methods
}

/// The name of each statement in an `authentication` block. Secrets can carry a trailing `## SECRET-DATA` marker,
/// so only the first word is kept.
fn authentication_methods(auth: &ConfigLine) -> Vec<String> {
    active_children(auth)
        .filter_map(|c| c.text.split_whitespace().next())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::system::system;
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
    fn configuration_limits_are_optional() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert_eq!(system.max_configurations_on_flash, None);
        assert_eq!(system.max_configuration_rollbacks, None);
        assert!(!system.no_redirects);
    }

    #[test]
    fn extracts_system_flags() {
        let system = system(&juniper::parse_braces(
            "system {\n    no-multicast-echo;\n    no-ping-record-route;\n    no-ping-time-stamp;\n    auto-snapshot;\n    compress-configuration-files;\n    mirror-flash-on-disk;\n    switchover-on-routing-crash;\n    use-imported-time-zones;\n}\n",
        ))
        .unwrap();
        assert!(system.no_multicast_echo);
        assert!(system.no_ping_record_route);
        assert!(system.no_ping_time_stamp);
        assert!(system.auto_snapshot);
        assert!(system.compress_configuration_files);
        assert!(system.mirror_flash_on_disk);
        assert!(system.switchover_on_routing_crash);
        assert!(system.use_imported_time_zones);
    }

    #[test]
    fn system_flags_default_to_off() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(!system.no_multicast_echo);
        assert!(!system.auto_snapshot);
        assert!(!system.use_imported_time_zones);
    }

    #[test]
    fn no_system_is_none() {
        assert!(system(&juniper::parse_braces("interfaces {\n    lo0;\n}\n")).is_none());
    }

    #[test]
    fn diag_port_and_pic_console_report_only_the_method() {
        let system = system(&juniper::parse_braces(
            r#"
system {
    diag-port-authentication encrypted-password "$6$salt$hash"; ## SECRET-DATA
    pic-console-authentication {
        encrypted-password "$6$salt$other"; ## SECRET-DATA
    }
}
"#,
        ))
        .unwrap();
        assert_eq!(system.diag_port_authentication, ["encrypted-password"]);
        assert_eq!(system.pic_console_authentication, ["encrypted-password"]);
    }

    #[test]
    fn secret_settings_are_optional() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.master_password.is_none());
        assert!(system.diag_port_authentication.is_empty());
        assert!(system.pic_console_authentication.is_empty());
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
    fn authentication_settings_are_optional() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.authentication_order.is_empty());
        assert!(system.radius_servers.is_empty());
        assert!(system.tacplus_servers.is_empty());
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
    fn addressing_settings_are_optional() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.domain_search.is_empty());
        assert!(system.static_host_mappings.is_empty());
        assert!(system.backup_router.is_none());
        assert!(system.inet6_backup_router.is_none());
    }
}
