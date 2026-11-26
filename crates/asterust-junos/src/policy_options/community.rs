//! `policy-options { community <name> members ...; }`.

use crate::{active_children, list_values, split_keyword, split_name};
use asterust::config::ConfigLine;

/// A `community <name> members ...`, which may be a standard, extended or large community.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Community {
    pub name: String,
    pub members: Vec<String>,
    pub invert_match: bool,
}

/// A `community` statement written either on one line (`community X members 1:1;`) or as a block.
pub(super) fn parse_community(line: &ConfigLine) -> Option<Community> {
    let (name, rest) = split_name(line.arg("community")?);
    let (keyword, members) = split_keyword(rest);
    let mut community = Community {
        name: name.to_string(),
        members: if keyword == "members" {
            list_values(members)
        } else {
            Vec::new()
        },
        invert_match: false,
    };
    for child in active_children(line) {
        if let Some(members) = child.arg("members") {
            community.members.extend(list_values(members));
        } else if child.text == "invert-match" {
            community.invert_match = true;
        }
    }
    Some(community)
}
