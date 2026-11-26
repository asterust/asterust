//! `system { syslog { user <name> { ... } } }`.

use super::file::{targets, SyslogTarget};
use asterust::config::ConfigLine;

pub(super) fn users(syslog: &ConfigLine) -> Vec<SyslogTarget> {
    targets(syslog, "user")
}
