//! `system { syslog { ... } }`.

use crate::{active_children, arg_of, child_block, named_children};
use asterust::config::ConfigLine;

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

pub(super) fn parse_syslog(syslog: &ConfigLine) -> Syslog {
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

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

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
}
