//! `log-updown`
//! `protocols { mpls { log-updown { ... } } }`.
//!
//! # See also
//!
//! - [log-updown (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/log-updown-edit-protocols-mpls.html)

use crate::{child_block, has_leaf, toggle};
use asterust::config::ConfigLine;

/// The `log-updown` options of MPLS: LSP up/down logging and SNMP traps.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct MplsLogUpdown {
    /// `Some(true)` for `syslog`, `Some(false)` for `no-syslog`.
    pub syslog: Option<bool>,
    pub trap: bool,
    pub trap_path_down: bool,
    pub trap_path_up: bool,
    /// `no-trap { mpls-lsp-traps; }`.
    pub no_trap_mpls_lsp_traps: bool,
    /// `no-trap { rfc3812-traps; }`.
    pub no_trap_rfc3812_traps: bool,
}

pub(super) fn parse_log_updown(mpls: &ConfigLine) -> Option<MplsLogUpdown> {
    let log = child_block(mpls, "log-updown")?;
    let no_trap = child_block(log, "no-trap");
    let in_no_trap = |flag: &str| no_trap.is_some_and(|block| has_leaf(block, flag));
    Some(MplsLogUpdown {
        syslog: toggle(log, "syslog"),
        trap: has_leaf(log, "trap"),
        trap_path_down: has_leaf(log, "trap-path-down"),
        trap_path_up: has_leaf(log, "trap-path-up"),
        no_trap_mpls_lsp_traps: in_no_trap("mpls-lsp-traps"),
        no_trap_rfc3812_traps: in_no_trap("rfc3812-traps"),
    })
}
