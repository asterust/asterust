//! `as-path`
//! `policy-options { as-path <name> <regular-expression>; }`.
//!
//! # See also
//!
//! - [as-path (Policy Options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/as-path-edit-policy-options.html)

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
