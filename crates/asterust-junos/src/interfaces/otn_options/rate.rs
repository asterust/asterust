//! `rate`
//! `interfaces { <name> { otn-options { rate { ... } } } }`.
//!
//! # See also
//!
//! - [rate](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/rate-edit-interfaces-otn-options.html)

use crate::{has_leaf, toggle};
use asterust::config::ConfigLine;

/// The `rate` of an OTN interface.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct OtnRate {
    /// `Some(true)` for `fixed-stuff-bytes`, `Some(false)` for `no-fixed-stuff-bytes`.
    pub fixed_stuff_bytes: Option<bool>,
    /// `Some(true)` for `pass-through`, `Some(false)` for `no-pass-through`.
    pub pass_through: Option<bool>,
    pub otu4: bool,
    pub oc192: bool,
}

pub(super) fn parse_rate(rate: &ConfigLine) -> OtnRate {
    OtnRate {
        fixed_stuff_bytes: toggle(rate, "fixed-stuff-bytes"),
        pass_through: toggle(rate, "pass-through"),
        otu4: has_leaf(rate, "otu4"),
        oc192: has_leaf(rate, "oc192"),
    }
}
