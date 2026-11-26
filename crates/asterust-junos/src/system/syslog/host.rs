//! `host`
//! `system { syslog { host <address> { ... } } }`.
//!
//! # See also
//!
//! - [host (System)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/host-edit-system.html)

use super::{syslog_rules, SyslogRule};
use crate::{arg_of, has_leaf};
use asterust::config::ConfigLine;

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
    pub allow_duplicates: bool,
    pub explicit_priority: bool,
    pub exclude_hostname: bool,
    /// The regular expression from `match`.
    pub match_regex: Option<String>,
}

pub(super) fn parse_host(address: &str, host: &ConfigLine) -> SyslogHost {
    SyslogHost {
        address: address.to_string(),
        rules: syslog_rules(host),
        port: arg_of(host, "port").map(str::to_string),
        source_address: arg_of(host, "source-address").map(str::to_string),
        routing_instance: arg_of(host, "routing-instance").map(str::to_string),
        facility_override: arg_of(host, "facility-override").map(str::to_string),
        log_prefix: arg_of(host, "log-prefix").map(str::to_string),
        allow_duplicates: has_leaf(host, "allow-duplicates"),
        explicit_priority: has_leaf(host, "explicit-priority"),
        exclude_hostname: has_leaf(host, "exclude-hostname"),
        match_regex: arg_of(host, "match").map(str::to_string),
    }
}
