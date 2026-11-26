//! Extracts the firewall `filter` applied to a family.
//!
//! Configuration path: `interfaces { <name> { unit <n> { family <name> { filter { ... } } } } }`.
//!
//! # See also
//!
//! - [filter (Applying to a Logical Interface)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/filter-edit-interfaces-unit-family.html)

use crate::{arg_of, args_of, child_block, list_values};
use asterust::config::ConfigLine;

/// The firewall filters applied to a family.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct FamilyFilter {
    pub group: Option<String>,
    pub input: Option<String>,
    pub output: Option<String>,
    pub input_list: Vec<String>,
    pub output_list: Vec<String>,
}

pub(super) fn parse_filter(family: &ConfigLine) -> Option<FamilyFilter> {
    let filter = child_block(family, "filter")?;
    let list = |keyword: &str| args_of(filter, keyword).into_iter().flat_map(list_values).collect();
    Some(FamilyFilter {
        group: arg_of(filter, "group").map(str::to_string),
        input: arg_of(filter, "input").map(str::to_string),
        output: arg_of(filter, "output").map(str::to_string),
        input_list: list("input-list"),
        output_list: list("output-list"),
    })
}
