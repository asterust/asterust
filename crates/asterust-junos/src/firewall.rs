//! `firewall { ... }`: stateless filters and policers.

use crate::{
    active_children, arg_of, child_block, list_values, named_children, split_keyword, statements, top_level, unquote,
};
use asterust::config::{Config, ConfigLine};

/// One `from` match condition, e.g. `destination-port bgp` or `source-address { 10.0.0.0/8; }`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct MatchCondition {
    pub name: String,
    /// Values as written, including any trailing modifier such as `except`; empty for a flag like `first-fragment`.
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

/// A `policer <name>`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Policer {
    pub name: String,
    pub bandwidth_limit: Option<String>,
    pub bandwidth_percent: Option<String>,
    pub burst_size_limit: Option<String>,
    /// `then discard`.
    pub discard: bool,
    pub forwarding_class: Option<String>,
    pub loss_priority: Option<String>,
}

/// The top-level `firewall` configuration. `simple-filter`, `service-filter` and `prefix-action` are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Firewall {
    pub filters: Vec<FirewallFilter>,
    pub policers: Vec<Policer>,
}

/// The top-level `firewall` block, or `None` if there is none.
pub fn firewall(config: &Config) -> Option<Firewall> {
    let root = top_level(config, "firewall")?;
    let filters = named_children(root, "family")
        .flat_map(|(family, family_line)| {
            named_children(family_line, "filter").map(move |(name, filter)| parse_filter(family, name, filter))
        })
        .collect();
    Some(Firewall {
        filters,
        policers: named_children(root, "policer")
            .map(|(name, line)| parse_policer(name, line))
            .collect(),
    })
}

fn parse_filter(family: &str, name: &str, line: &ConfigLine) -> FirewallFilter {
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

fn parse_policer(name: &str, line: &ConfigLine) -> Policer {
    let limit = |keyword: &str| {
        child_block(line, "if-exceeding")
            .and_then(|block| arg_of(block, keyword))
            .map(str::to_string)
    };
    let mut policer = Policer {
        name: name.to_string(),
        bandwidth_limit: limit("bandwidth-limit"),
        bandwidth_percent: limit("bandwidth-percent"),
        burst_size_limit: limit("burst-size-limit"),
        discard: false,
        forwarding_class: None,
        loss_priority: None,
    };
    for statement in statements(line, "then") {
        let (action, arg) = split_keyword(statement);
        match action {
            "discard" => policer.discard = true,
            "forwarding-class" => policer.forwarding_class = Some(unquote(arg).to_string()),
            "loss-priority" => policer.loss_priority = Some(unquote(arg).to_string()),
            _ => {}
        }
    }
    policer
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::juniper;

    fn parse(text: &str) -> Firewall {
        firewall(&juniper::parse_braces(text)).expect("fixture has a firewall block")
    }

    // Adapted from Juniper's BGP User Guide "TCP Access Restriction for BGP" (Device E and the prefix-list example);
    // see the README "Sources".
    const DOC_BGP179_PEERS: &str = r#"
firewall {
    family inet {
        filter filter_bgp179 {
            term 1 {
                from {
                    source-address {
                        10.10.10.2/32;
                        10.10.10.6/32;
                    }
                    destination-port bgp;
                }
                then accept;
            }
            term 2 {
                then {
                    reject;
                }
            }
        }
    }
}
"#;

    const DOC_BGP179_PREFIX_LIST: &str = r#"
firewall {
    family inet {
        filter filter_bgp179 {
            term 1 {
                from {
                    source-address {
                        0.0.0.0/0;
                    }
                    source-prefix-list {
                        plist_bgp179 except;
                    }
                    destination-port bgp;
                }
                then {
                    reject;
                }
            }
            term 2 {
                then {
                    accept;
                }
            }
        }
    }
}
"#;

    #[test]
    fn doc_filter_with_address_block_and_leaf_then() {
        let firewall = parse(DOC_BGP179_PEERS);
        assert_eq!(firewall.filters.len(), 1);
        let filter = &firewall.filters[0];
        assert_eq!(filter.family, "inet");
        assert_eq!(filter.name, "filter_bgp179");
        assert_eq!(filter.terms.len(), 2);

        let term = &filter.terms[0];
        assert_eq!(term.name, "1");
        assert_eq!(
            term.condition("source-address").unwrap().values,
            ["10.10.10.2/32", "10.10.10.6/32"]
        );
        assert_eq!(term.condition("destination-port").unwrap().values, ["bgp"]);
        assert_eq!(term.then.action, Some(FilterAction::Accept));
    }

    #[test]
    fn doc_term_without_from_and_block_then() {
        let filter = &parse(DOC_BGP179_PEERS).filters[0];
        assert!(filter.terms[1].from.is_empty());
        assert_eq!(filter.terms[1].then.action, Some(FilterAction::Reject));
    }

    #[test]
    fn doc_prefix_list_with_except_modifier() {
        let filter = &parse(DOC_BGP179_PREFIX_LIST).filters[0];
        let term = &filter.terms[0];
        assert_eq!(term.condition("source-address").unwrap().values, ["0.0.0.0/0"]);
        assert_eq!(
            term.condition("source-prefix-list").unwrap().values,
            ["plist_bgp179 except"]
        );
        assert_eq!(term.then.action, Some(FilterAction::Reject));
        assert_eq!(filter.terms[1].then.action, Some(FilterAction::Accept));
    }

    // Not from documentation: covers statements the Juniper examples do not use. See the README "Sources".
    const CONFIG: &str = r#"
firewall {
    family inet {
        filter PROTECT-RE {
            term ICMP {
                from {
                    protocol icmp;
                    icmp-type [ echo-request echo-reply ];
                    first-fragment;
                }
                then {
                    policer ICMP-LIMIT;
                    count icmp-packets;
                    log;
                    accept;
                }
            }
            term SSH {
                from destination-port ssh;
                then {
                    forwarding-class network-control;
                    loss-priority low;
                    next term;
                }
            }
            inactive: term OLD {
                then discard;
            }
            term DEFAULT {
                then {
                    syslog;
                    discard;
                }
            }
        }
    }
    family inet6 {
        filter V6 {
            term ALL {
                then accept;
            }
        }
    }
    policer ICMP-LIMIT {
        if-exceeding {
            bandwidth-limit 1m;
            burst-size-limit 15k;
        }
        then discard;
    }
    policer PERCENT {
        if-exceeding {
            bandwidth-percent 10;
            burst-size-limit 10k;
        }
        then {
            forwarding-class best-effort;
            loss-priority high;
        }
    }
}
"#;

    #[test]
    fn lists_flags_and_nonterminating_actions() {
        let firewall = parse(CONFIG);
        let names: Vec<_> = firewall
            .filters
            .iter()
            .map(|f| (f.family.as_str(), f.name.as_str()))
            .collect();
        assert_eq!(names, [("inet", "PROTECT-RE"), ("inet6", "V6")]);

        let icmp = &firewall.filters[0].terms[0];
        assert_eq!(icmp.condition("protocol").unwrap().values, ["icmp"]);
        assert_eq!(
            icmp.condition("icmp-type").unwrap().values,
            ["echo-request", "echo-reply"]
        );
        assert!(icmp.condition("first-fragment").unwrap().values.is_empty());
        assert_eq!(icmp.then.policer.as_deref(), Some("ICMP-LIMIT"));
        assert_eq!(icmp.then.count.as_deref(), Some("icmp-packets"));
        assert!(icmp.then.log);
        assert!(!icmp.then.syslog);
    }

    #[test]
    fn leaf_from_next_term_and_skipped_inactive_term() {
        let terms = &parse(CONFIG).filters[0].terms;
        let names: Vec<_> = terms.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, ["ICMP", "SSH", "DEFAULT"]);

        let ssh = &terms[1];
        assert_eq!(ssh.condition("destination-port").unwrap().values, ["ssh"]);
        assert_eq!(ssh.then.action, Some(FilterAction::NextTerm));
        assert_eq!(ssh.then.forwarding_class.as_deref(), Some("network-control"));
        assert_eq!(ssh.then.loss_priority.as_deref(), Some("low"));

        assert_eq!(terms[2].then.action, Some(FilterAction::Discard));
        assert!(terms[2].then.syslog);
    }

    #[test]
    fn extracts_policers() {
        let policers = parse(CONFIG).policers;
        assert_eq!(policers.len(), 2);

        assert_eq!(policers[0].name, "ICMP-LIMIT");
        assert_eq!(policers[0].bandwidth_limit.as_deref(), Some("1m"));
        assert_eq!(policers[0].burst_size_limit.as_deref(), Some("15k"));
        assert_eq!(policers[0].bandwidth_percent, None);
        assert!(policers[0].discard);

        assert_eq!(policers[1].bandwidth_percent.as_deref(), Some("10"));
        assert!(!policers[1].discard);
        assert_eq!(policers[1].forwarding_class.as_deref(), Some("best-effort"));
        assert_eq!(policers[1].loss_priority.as_deref(), Some("high"));
    }

    #[test]
    fn no_firewall_is_none() {
        assert!(firewall(&juniper::parse_braces("system {\n host-name r1;\n}\n")).is_none());
    }
}
