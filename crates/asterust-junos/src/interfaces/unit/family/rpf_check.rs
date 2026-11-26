//! `rpf-check`
//! `interfaces { <name> { unit <n> { family <name> { rpf-check { ... } } } } }`.
//!
//! # See also
//!
//! - [rpf-check](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/rpf-check-edit-interfaces.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The `rpf-check` unicast reverse-path-forwarding settings; both fields are `None` for a bare
/// `rpf-check;`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct RpfCheck {
    pub fail_filter: Option<String>,
    /// `strict` or `loose`.
    pub mode: Option<String>,
}

pub(super) fn parse_rpf_check(family: &ConfigLine) -> Option<RpfCheck> {
    let words = statement_words(family, "rpf-check")?;
    Some(RpfCheck {
        fail_filter: value_after(&words, "fail-filter"),
        mode: value_after(&words, "mode"),
    })
}
