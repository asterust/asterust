//! `system { ... }`.

mod accounting;
mod addressing;
mod archival;
mod auth;
mod internet;
mod location;
mod login;
mod ntp;
mod ports;
mod services;
mod syslog;

pub use accounting::{Accounting, AccountingDestination};
pub use addressing::{BackupRouter, NameServer, StaticHostMapping};
pub use archival::{Archival, ArchivalConfiguration};
pub use auth::{RadiusServer, RootAuthentication, TacplusServer};
pub use internet::{IcmpRateLimit, InternetOptions};
pub use location::Location;
pub use login::{Login, LoginClass, LoginPassword, LoginRetryOptions, LoginUser};
pub use ntp::{Ntp, NtpAuthenticationKey, NtpHost};
pub use ports::{Auxiliary, Console, Ports};
pub use services::{OutboundSshClient, OutboundSshServer, Services, SshService};
pub use syslog::{Syslog, SyslogArchive, SyslogHost, SyslogRule, SyslogStructuredData, SyslogTarget};

use crate::{arg_of, args_of, child_block, has_leaf, list_values, top_level};
use accounting::parse_accounting;
use addressing::{name_servers, parse_backup_router, static_host_mappings};
use archival::parse_archival;
use asterust::config::Config;
use auth::{radius_servers, root_authentication, tacplus_servers};
use internet::parse_internet_options;
use location::parse_location;
use login::parse_login;
use ntp::parse_ntp;
use ports::parse_ports;
use services::parse_services;
use syslog::parse_syslog;

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
    pub archival: Option<Archival>,
    pub accounting: Option<Accounting>,
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
        archival: child_block(system, "archival").map(parse_archival),
        accounting: child_block(system, "accounting").map(parse_accounting),
        root_authentication: root_authentication(system),
        name_servers: name_servers(system),
        ntp: child_block(system, "ntp").map(parse_ntp),
        syslog: child_block(system, "syslog").map(parse_syslog),
        login: child_block(system, "login").map(parse_login),
        services: child_block(system, "services").map(parse_services),
    })
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
    fn no_system_is_none() {
        assert!(system(&juniper::parse_braces("interfaces {\n    lo0;\n}\n")).is_none());
    }
}
