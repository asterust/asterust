//! `policy-options { policy-statement <name> { ... } }`.

mod term;

pub use term::{
    CommunityAction, CommunityOperation, PolicyAction, PolicyActions, PolicyConditions, PolicyTerm, RouteFilter,
};

use crate::{active_children, named_children};
use asterust::config::ConfigLine;
use term::parse_term;

/// A `policy-statement <name>`. An unnamed term is placed last.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PolicyStatement {
    pub name: String,
    pub terms: Vec<PolicyTerm>,
}

pub(super) fn parse_policy(name: &str, line: &ConfigLine) -> PolicyStatement {
    let mut terms: Vec<PolicyTerm> = named_children(line, "term")
        .map(|(term_name, term)| parse_term(Some(term_name), term))
        .collect();
    let has_unnamed_term = active_children(line).any(|c| {
        ["from", "to", "then"]
            .iter()
            .any(|k| c.text == *k || c.arg(k).is_some())
    });
    if has_unnamed_term {
        terms.push(parse_term(None, line));
    }
    PolicyStatement {
        name: name.to_string(),
        terms,
    }
}
