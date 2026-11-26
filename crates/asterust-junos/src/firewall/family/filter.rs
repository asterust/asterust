//! Extracts the `filter` entries of a firewall `family`, with their terms.
//!
//! Configuration path: `firewall { family <family> { filter <name> { term <name> { from { ... } then { ... } } } } }`.
//!
//! # See also
//!
//! - [filter (Configuring)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/filter-edit-firewall.html)
//! - [term (Firewall Filter)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/term-edit-firewall.html)

use crate::{active_children, list_values, named_children, split_keyword, statements, unquote};
use asterust::config::ConfigLine;

/// One `from` match condition, e.g. `destination-port bgp` or `source-address { 10.0.0.0/8; }`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct MatchCondition {
    pub name: String,
    /// Values as written, including any trailing modifier such as `except`; empty for a flag like
    /// `first-fragment`.
    pub values: Vec<String>,
}

/// The terminating action of a filter term. A term without one accepts matching packets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FilterAction {
    Accept,
    Discard,
    Reject,
    NextTerm,
}

/// The actions of a term's `then` statement. Actions not listed here are ignored.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct FilterActions {
    pub action: Option<FilterAction>,
    /// Counter name from `count`.
    pub count: Option<String>,
    pub policer: Option<String>,
    pub forwarding_class: Option<String>,
    pub loss_priority: Option<String>,
    pub log: bool,
    pub syslog: bool,
}

/// A `term` of a filter.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FilterTerm {
    pub name: String,
    pub from: Vec<MatchCondition>,
    pub then: FilterActions,
}

impl FilterTerm {
    /// The `from` condition called `name`, e.g. `"destination-port"`.
    pub fn condition(&self, name: &str) -> Option<&MatchCondition> {
        self.from.iter().find(|c| c.name == name)
    }
}

/// A `filter <name>` under `family <family>`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FirewallFilter {
    /// Address family, e.g. `"inet"`, `"inet6"`, `"bridge"`.
    pub family: String,
    pub name: String,
    pub terms: Vec<FilterTerm>,
}

pub(super) fn parse_filter(family: &str, name: &str, line: &ConfigLine) -> FirewallFilter {
    FirewallFilter {
        family: family.to_string(),
        name: name.to_string(),
        terms: named_children(line, "term")
            .map(|(term_name, term)| FilterTerm {
                name: term_name.to_string(),
                from: parse_conditions(term),
                then: parse_actions(term),
            })
            .collect(),
    }
}

/// Conditions written either as `from x;` or inside a `from { ... }` block.
fn parse_conditions(term: &ConfigLine) -> Vec<MatchCondition> {
    let mut found = Vec::new();
    for child in active_children(term) {
        if child.text == "from" {
            found.extend(active_children(child).map(parse_condition));
        } else if let Some(statement) = child.arg("from") {
            found.push(condition_from(statement, &[]));
        }
    }
    found
}

fn parse_condition(line: &ConfigLine) -> MatchCondition {
    condition_from(&line.text, &line.children)
}

fn condition_from(statement: &str, children: &[ConfigLine]) -> MatchCondition {
    let (name, arg) = split_keyword(statement);
    // A bare value keeps its modifier (`10.0.0.0/8 except`); only a bracketed list is split.
    let mut values = if arg.is_empty() {
        Vec::new()
    } else if arg.starts_with('[') {
        list_values(arg)
    } else {
        vec![unquote(arg).to_string()]
    };
    values.extend(
        children
            .iter()
            .filter(|c| !c.text.starts_with("inactive:"))
            .map(|c| c.text.clone()),
    );
    MatchCondition {
        name: name.to_string(),
        values,
    }
}

fn parse_actions(term: &ConfigLine) -> FilterActions {
    let mut actions = FilterActions::default();
    for statement in statements(term, "then") {
        let (action, arg) = split_keyword(statement);
        let value = || Some(unquote(arg).to_string());
        match (action, arg) {
            ("accept", _) => actions.action = Some(FilterAction::Accept),
            ("discard", _) => actions.action = Some(FilterAction::Discard),
            ("reject", _) => actions.action = Some(FilterAction::Reject),
            ("next", "term") => actions.action = Some(FilterAction::NextTerm),
            ("count", _) => actions.count = value(),
            ("policer", _) => actions.policer = value(),
            ("forwarding-class", _) => actions.forwarding_class = value(),
            ("loss-priority", _) => actions.loss_priority = value(),
            ("log", "") => actions.log = true,
            ("syslog", "") => actions.syslog = true,
            _ => {}
        }
    }
    actions
}
