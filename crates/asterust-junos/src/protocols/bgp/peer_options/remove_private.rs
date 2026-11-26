//! `remove-private`
//! `protocols { bgp { group <name> { remove-private [all] [replace] [nearest] [no-peer-loop-check]; } } }`.
//!
//! # See also
//!
//! - [remove-private](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/remove-private-edit-protocols-bgp.html)

use crate::statement_words;
use asterust::config::ConfigLine;

/// The `remove-private` statement, which strips private AS numbers from advertised AS paths.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct RemovePrivate {
    pub all: bool,
    pub replace: bool,
    pub nearest: bool,
    pub no_peer_loop_check: bool,
}

/// Reads a bare `remove-private;`, inline words or a block.
pub(super) fn parse_remove_private(line: &ConfigLine) -> Option<RemovePrivate> {
    let words = statement_words(line, "remove-private")?;
    Some(RemovePrivate {
        all: words.contains(&"all"),
        replace: words.contains(&"replace"),
        nearest: words.contains(&"nearest"),
        no_peer_loop_check: words.contains(&"no-peer-loop-check"),
    })
}
