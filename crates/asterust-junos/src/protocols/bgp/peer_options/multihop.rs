//! `multihop`
//! `protocols { bgp { group <name> { multihop { ttl <n>; no-nexthop-change; } } } }`.
//!
//! # See also
//!
//! - [multihop](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/multihop-edit-protocols-bgp.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The `multihop` statement for sessions to peers that are not directly connected.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Multihop {
    pub ttl: Option<String>,
    pub no_nexthop_change: bool,
}

/// Reads the block form and the older `multihop <ttl>;` form.
pub(super) fn parse_multihop(line: &ConfigLine) -> Option<Multihop> {
    let words = statement_words(line, "multihop")?;
    let bare_ttl = words
        .first()
        .filter(|word| word.chars().all(|c| c.is_ascii_digit()))
        .map(|word| word.to_string());
    Some(Multihop {
        ttl: value_after(&words, "ttl").or(bare_ttl),
        no_nexthop_change: words.contains(&"no-nexthop-change"),
    })
}
