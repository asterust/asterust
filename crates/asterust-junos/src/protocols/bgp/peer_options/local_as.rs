//! `local-as`
//! `protocols { bgp { group <name> { local-as <as> [loops <n>] [private | alias] [no-prepend-global-as]; } } }`.
//!
//! # See also
//!
//! - [local-as](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/local-as-edit-protocols-bgp.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The `local-as` statement: the AS number the router presents to a peer, and how it is prepended.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LocalAs {
    pub autonomous_system: Option<String>,
    /// How many times the local AS may appear in the AS path.
    pub loops: Option<String>,
    pub private: bool,
    pub alias: bool,
    pub no_prepend_global_as: bool,
}

pub(super) fn parse_local_as(line: &ConfigLine) -> Option<LocalAs> {
    let words = statement_words(line, "local-as")?;
    let first = words
        .first()
        .filter(|word| word.chars().next().is_some_and(|c| c.is_ascii_digit()));
    Some(LocalAs {
        autonomous_system: first.map(|word| word.to_string()),
        loops: value_after(&words, "loops"),
        private: words.contains(&"private"),
        alias: words.contains(&"alias"),
        no_prepend_global_as: words.contains(&"no-prepend-global-as"),
    })
}
