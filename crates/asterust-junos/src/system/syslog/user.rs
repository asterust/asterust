//! Extracts the `user` destinations of `system syslog`.
//!
//! Configuration path: `system { syslog { user <name> { ... } } }`.
//!
//! # See also
//!
//! - [user (System Logging)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/user-edit-system-syslog.html)

use super::file::{targets, SyslogTarget};
use asterust::config::ConfigLine;

pub(super) fn users(syslog: &ConfigLine) -> Vec<SyslogTarget> {
    targets(syslog, "user")
}
