//! `xnm-clear-text`
//! `system { services { xnm-clear-text { ... } } }`.
//!
//! # See also
//!
//! - [xnm-clear-text](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/xnm-clear-text-edit-system.html)

use crate::arg_of;
use asterust::config::ConfigLine;

/// The `xnm-clear-text` block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct XnmClearText {
    pub connection_limit: Option<String>,
    pub rate_limit: Option<String>,
}

pub(super) fn parse_xnm_clear_text(xnm: &ConfigLine) -> XnmClearText {
    XnmClearText {
        connection_limit: arg_of(xnm, "connection-limit").map(str::to_string),
        rate_limit: arg_of(xnm, "rate-limit").map(str::to_string),
    }
}
