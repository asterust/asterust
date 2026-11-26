//! `policy-options { as-path <name> <regular-expression>; }`.

use crate::{split_name, unquote};
use asterust::config::ConfigLine;

/// An `as-path <name> <regular-expression>`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AsPath {
    pub name: String,
    pub regex: String,
}

pub(super) fn parse_as_path(line: &ConfigLine) -> Option<AsPath> {
    let (name, regex) = split_name(line.arg("as-path")?);
    Some(AsPath {
        name: name.to_string(),
        regex: unquote(regex).to_string(),
    })
}
