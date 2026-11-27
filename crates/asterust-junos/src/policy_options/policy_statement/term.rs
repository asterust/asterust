//! `term`
//! `policy-options { policy-statement <name> { term <name> { from { ... } to { ... } then { ... } } } }`.
//!
//! A policy written without `term` statements is a single unnamed term. Each term carries `from`,
//! `to` and `then` statements.
//!
//! # See also
//!
//! - [policy-statement](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/policy-statement-edit-policy-options.html)
//! - [load-balance (ecmp)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/load-balance-edit-policy-options-policy-statement-then.html)
//! - [route-filter](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/route-filter-edit-policy-options.html)

use crate::{list_values, split_keyword, split_name, statements, unquote};
use asterust::config::ConfigLine;

/// A `route-filter <prefix> <match-type>` condition.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct RouteFilter {
    pub prefix: String,
    /// Everything after the prefix, e.g. `"exact"`, `"orlonger"`, `"upto /24"`.
    pub match_type: Option<String>,
}

/// The match conditions of a term's `from` or `to` statement. Conditions not listed here are
/// ignored.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct PolicyConditions {
    pub protocols: Vec<String>,
    pub families: Vec<String>,
    pub communities: Vec<String>,
    pub as_paths: Vec<String>,
    pub prefix_lists: Vec<String>,
    /// The raw `prefix-list-filter` argument, e.g. `"CUSTOMERS orlonger"`.
    pub prefix_list_filters: Vec<String>,
    pub route_filters: Vec<RouteFilter>,
    pub neighbors: Vec<String>,
    /// Subroutine policies from `policy`.
    pub policies: Vec<String>,
}

/// The flow-control action ending a term.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PolicyAction {
    Accept,
    Reject,
    NextTerm,
    NextPolicy,
}

/// How a `then community` action changes the route's communities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CommunityOperation {
    Add,
    Set,
    Delete,
}

/// A `then community (add | set | delete) <name>` action.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct CommunityAction {
    pub operation: CommunityOperation,
    pub community: String,
}

/// A `then srv6 locator <name> <sid>` action, which selects the SRv6 service SID that BGP advertises for the route.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PolicySrv6Action {
    pub locator: String,
    /// `end-dt4-sid`, `micro-dt46-sid` and the like.
    pub sid_type: Option<String>,
    /// The static SID value, if given.
    pub sid_value: Option<String>,
}

/// The actions of a term's `then` statement. Actions not listed here are ignored.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct PolicyActions {
    pub action: Option<PolicyAction>,
    /// Everything after `metric`, e.g. `"63"` or `"add 10"`.
    pub metric: Option<String>,
    pub local_preference: Option<String>,
    pub next_hop: Option<String>,
    pub as_path_prepend: Option<String>,
    pub communities: Vec<CommunityAction>,
    /// Everything after `load-balance`, e.g. `"per-packet"`. A `load-balance { ...; }` block is not read.
    pub load_balance: Option<String>,
    /// The `resolution-map` to apply, which is defined under `policy-options`.
    pub resolution_map: Option<String>,
    pub multipath_resolve: bool,
    pub srv6: Option<PolicySrv6Action>,
}

/// One term of a policy statement.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PolicyTerm {
    /// `None` for the unnamed term, which is the only one in a policy written without `term`
    /// statements.
    pub name: Option<String>,
    pub from: PolicyConditions,
    pub to: PolicyConditions,
    pub then: PolicyActions,
}

pub(super) fn parse_term(name: Option<&str>, line: &ConfigLine) -> PolicyTerm {
    PolicyTerm {
        name: name.map(str::to_string),
        from: parse_conditions(line, "from"),
        to: parse_conditions(line, "to"),
        then: parse_actions(line),
    }
}

fn parse_conditions(line: &ConfigLine, keyword: &str) -> PolicyConditions {
    let mut conditions = PolicyConditions::default();
    for statement in statements(line, keyword) {
        let (condition, arg) = split_keyword(statement);
        match condition {
            "protocol" => conditions.protocols.extend(list_values(arg)),
            "family" => conditions.families.push(unquote(arg).to_string()),
            "community" => conditions.communities.extend(list_values(arg)),
            "as-path" => conditions.as_paths.extend(list_values(arg)),
            "prefix-list" => conditions.prefix_lists.extend(list_values(arg)),
            "prefix-list-filter" => conditions.prefix_list_filters.push(arg.to_string()),
            "route-filter" => conditions.route_filters.push(parse_route_filter(arg)),
            "neighbor" => conditions.neighbors.push(unquote(arg).to_string()),
            "policy" => conditions.policies.push(unquote(arg).to_string()),
            _ => {}
        }
    }
    conditions
}

fn parse_route_filter(arg: &str) -> RouteFilter {
    let (prefix, match_type) = split_keyword(arg);
    RouteFilter {
        prefix: prefix.to_string(),
        match_type: Some(match_type).filter(|m| !m.is_empty()).map(str::to_string),
    }
}

fn parse_actions(line: &ConfigLine) -> PolicyActions {
    let mut actions = PolicyActions::default();
    for statement in statements(line, "then") {
        let (action, arg) = split_keyword(statement);
        let value = || Some(unquote(arg).to_string());
        match (action, arg) {
            ("accept", "") => actions.action = Some(PolicyAction::Accept),
            ("reject", "") => actions.action = Some(PolicyAction::Reject),
            ("next", "term") => actions.action = Some(PolicyAction::NextTerm),
            ("next", "policy") => actions.action = Some(PolicyAction::NextPolicy),
            ("metric", _) => actions.metric = value(),
            ("local-preference", _) => actions.local_preference = value(),
            ("next-hop", _) => actions.next_hop = value(),
            ("as-path-prepend", _) => actions.as_path_prepend = value(),
            ("community", _) => actions.communities.extend(parse_community_action(arg)),
            ("load-balance", _) => actions.load_balance = Some(arg.to_string()).filter(|a| !a.is_empty()),
            ("resolution-map", _) => actions.resolution_map = value(),
            ("multipath-resolve", "") => actions.multipath_resolve = true,
            ("srv6", _) => actions.srv6 = parse_srv6_action(arg),
            _ => {}
        }
    }
    actions
}

fn parse_srv6_action(arg: &str) -> Option<PolicySrv6Action> {
    let (locator, sid) = split_name(arg.strip_prefix("locator")?.trim());
    let (sid_type, sid_value) = split_keyword(sid);
    Some(PolicySrv6Action {
        locator: unquote(locator).to_string(),
        sid_type: Some(sid_type).filter(|t| !t.is_empty()).map(str::to_string),
        sid_value: Some(sid_value).filter(|v| !v.is_empty()).map(str::to_string),
    })
}

fn parse_community_action(arg: &str) -> Option<CommunityAction> {
    let (operation, community) = split_name(arg);
    let operation = match operation {
        "add" => CommunityOperation::Add,
        "set" => CommunityOperation::Set,
        "delete" => CommunityOperation::Delete,
        _ => return None,
    };
    Some(CommunityAction {
        operation,
        community: unquote(community).to_string(),
    })
}
