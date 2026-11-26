//! `firewall`
//! `firewall { ... }`.
//!
//! # See also
//!
//! - [`[edit firewall]` hierarchy](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/topic-map/hierarchy-edit-firewall.html)

mod family;
mod policer;

pub use family::{FilterAction, FilterActions, FilterTerm, FirewallFilter, MatchCondition};
pub use policer::Policer;

use crate::{named_children, top_level};
use asterust::config::Config;
use family::filters;
use policer::parse_policer;

/// The top-level `firewall` configuration. `simple-filter`, `service-filter` and `prefix-action`
/// are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Firewall {
    pub filters: Vec<FirewallFilter>,
    pub policers: Vec<Policer>,
}

/// Returns the top-level `firewall` block, or `None` if there is none.
///
/// # Examples
///
/// ```
/// use asterust::config::juniper;
/// use asterust_junos::firewall::firewall;
///
/// let config = juniper::parse_braces("firewall {\n    policer LIMIT {\n        if-exceeding {\n            bandwidth-limit 1m;\n        }\n    }\n}\n");
/// let firewall = firewall(&config).unwrap();
/// assert_eq!(firewall.policers[0].name, "LIMIT");
/// assert_eq!(firewall.policers[0].bandwidth_limit.as_deref(), Some("1m"));
/// ```
pub fn firewall(config: &Config) -> Option<Firewall> {
    let root = top_level(config, "firewall")?;
    Some(Firewall {
        filters: filters(root),
        policers: named_children(root, "policer")
            .map(|(name, line)| parse_policer(name, line))
            .collect(),
    })
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
