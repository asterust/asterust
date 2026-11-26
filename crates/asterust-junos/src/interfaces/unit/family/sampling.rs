//! `interfaces { <name> { unit <n> { family <name> { sampling { ... } } } } }`.

use crate::statement_words;
use asterust::config::ConfigLine;

/// The directions in which a family is sampled.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct FamilySampling {
    pub input: bool,
    pub output: bool,
}

pub(super) fn parse_sampling(family: &ConfigLine) -> Option<FamilySampling> {
    let words = statement_words(family, "sampling")?;
    Some(FamilySampling {
        input: words.contains(&"input"),
        output: words.contains(&"output"),
    })
}
