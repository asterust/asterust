//! `policer`
//! `interfaces { <name> { unit <n> { family <name> { policer { ... } } } } }`.
//!
//! # See also
//!
//! - [policer (Applying to a Logical Interface)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/policer-edit-interfaces-unit-family.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The policers applied to a family.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct FamilyPolicer {
    pub arp: Option<String>,
    pub input: Option<String>,
    pub output: Option<String>,
}

pub(super) fn parse_policer(family: &ConfigLine) -> Option<FamilyPolicer> {
    let words = statement_words(family, "policer")?;
    Some(FamilyPolicer {
        arp: value_after(&words, "arp"),
        input: value_after(&words, "input"),
        output: value_after(&words, "output"),
    })
}
