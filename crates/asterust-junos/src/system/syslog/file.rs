//! `file`
//! `system { syslog { file <name> { ... } } }`.
//!
//! [`SyslogTarget`](crate::system::SyslogTarget) is shared with the `user` destinations.
//!
//! # See also
//!
//! - [file (System Logging)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/file-edit-system-syslog.html)

use super::archive::{parse_archive, SyslogArchive};
use super::{syslog_rules, SyslogRule};
use crate::{arg_of, child_block, has_leaf, named_children};
use asterust::config::ConfigLine;

/// A `structured-data` statement on a syslog file.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SyslogStructuredData {
    /// Omit the message text.
    pub brief: bool,
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
    pub allow_duplicates: bool,
    /// The regular expression from `match`.
    pub match_regex: Option<String>,
    /// Only files have `explicit-priority`.
    pub explicit_priority: bool,
    /// Only files have `structured-data`.
    pub structured_data: Option<SyslogStructuredData>,
}

pub(super) fn targets(syslog: &ConfigLine, keyword: &str) -> Vec<SyslogTarget> {
    named_children(syslog, keyword)
        .map(|(name, target)| SyslogTarget {
            name: name.to_string(),
            rules: syslog_rules(target),
            archive: parse_archive(target),
            allow_duplicates: has_leaf(target, "allow-duplicates"),
            match_regex: arg_of(target, "match").map(str::to_string),
            explicit_priority: has_leaf(target, "explicit-priority"),
            structured_data: child_block(target, "structured-data").map(|data| SyslogStructuredData {
                brief: has_leaf(data, "brief"),
            }),
        })
        .collect()
}

pub(super) fn files(syslog: &ConfigLine) -> Vec<SyslogTarget> {
    targets(syslog, "file")
}
