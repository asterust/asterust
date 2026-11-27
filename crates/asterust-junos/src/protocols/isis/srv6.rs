//! `srv6`
//! `protocols { isis { source-packet-routing { srv6 { locator <name> { ... } } } } }` and
//! `interface <name> { level <1|2> { srv6-adjacency-segment { ... } } }`.
//!
//! # See also
//!
//! - [srv6 (Routing Options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/srv6-edit-routing-options-source-packet-routing.html)
//! - [locator](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/locator-edit-routing-options-source-packet-routing-srv6.html)
//! - [Overview of SRv6 Network Programming in IS-IS Networks](https://networkingdocs.hpe.com/documentation/us/en/software/junos/is-is/segment-routing/topics/concept/isis-srv6-network-programming.html)
//! - [Example: Configuring SRv6 Network Programming in IS-IS Networks](https://networkingdocs.hpe.com/documentation/us/en/software/junos/segment-routing/topics/example/isis-configuring-srv6-network-programming.html)

use crate::routing_options::{flavors_of, sid_with_flavors};
use crate::{active_children, child_block, has_leaf, named_children, split_name};
use asterust::config::ConfigLine;

/// `protocols isis source-packet-routing srv6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct IsisSrv6 {
    pub transit_srh_insert: bool,
    pub locators: Vec<IsisSrv6Locator>,
}

/// A `locator <name>` advertised by IS-IS. Its prefix is under `routing-options source-packet-routing srv6`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct IsisSrv6Locator {
    pub name: String,
    pub algorithm: Option<String>,
    pub end_sid: Option<String>,
    /// `psp`, `usp` and `usd`.
    pub end_sid_flavors: Vec<String>,
    pub dynamic_end_sid: bool,
    /// `micro-node-sid`, which advertises the uN SID of a micro-SID locator.
    pub micro_node_sid: bool,
}

/// One `locator` of an `srv6-adjacency-segment`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct IsisSrv6AdjacencySegment {
    /// `protected` or `unprotected`.
    pub protection: Option<String>,
    pub locator: String,
    pub end_x_sid: Option<String>,
    /// `psp`, `usp` and `usd`.
    pub flavors: Vec<String>,
    /// `micro-adjacency-sid` is configured.
    pub micro_adjacency_sid: bool,
    /// The value of `micro-adjacency-sid`; without one a dynamic uA SID is allocated.
    pub micro_adjacency_sid_value: Option<String>,
    /// The system id of the `lan-neighbor` the segment is for.
    pub lan_neighbor: Option<String>,
}

pub(super) fn parse_srv6(spr: &ConfigLine) -> Option<IsisSrv6> {
    let srv6 = child_block(spr, "srv6")?;
    Some(IsisSrv6 {
        transit_srh_insert: has_leaf(srv6, "transit-srh-insert"),
        locators: named_children(srv6, "locator")
            .map(|(arg, locator)| parse_locator(arg, locator))
            .collect(),
    })
}

fn parse_locator(arg: &str, locator: &ConfigLine) -> IsisSrv6Locator {
    let (name, inline) = split_name(arg);
    let words: Vec<&str> = inline.split_whitespace().collect();
    let (end_sid, end_sid_flavors) = sid_with_flavors(inline, locator, "end-sid");
    let statement = |keyword: &str| {
        active_children(locator).any(|c| c.text == keyword || c.arg(keyword).is_some()) || words.contains(&keyword)
    };
    IsisSrv6Locator {
        name: name.to_string(),
        algorithm: active_children(locator)
            .find_map(|c| c.arg("algorithm"))
            .map(str::to_string)
            .or_else(|| {
                words
                    .iter()
                    .position(|w| *w == "algorithm")
                    .and_then(|i| words.get(i + 1))
                    .map(|w| w.to_string())
            }),
        end_sid,
        end_sid_flavors,
        dynamic_end_sid: statement("dynamic-end-sid"),
        micro_node_sid: statement("micro-node-sid"),
    }
}

/// The `srv6-adjacency-segment` statements under `parent`, a level or a `lan-neighbor`.
pub(super) fn parse_adjacency_segments(
    parent: &ConfigLine,
    lan_neighbor: Option<&str>,
) -> Vec<IsisSrv6AdjacencySegment> {
    let mut segments = Vec::new();
    for statement in active_children(parent)
        .filter(|c| c.text == "srv6-adjacency-segment" || c.arg("srv6-adjacency-segment").is_some())
    {
        if let Some(inline) = statement.arg("srv6-adjacency-segment") {
            segments.extend(inline_segment(inline, statement, lan_neighbor));
        }
        for child in active_children(statement) {
            let (protection, rest) = split_name(&child.text);
            if !matches!(protection, "protected" | "unprotected") {
                continue;
            }
            match rest.strip_prefix("locator") {
                Some(locator) => segments.push(segment(Some(protection), locator.trim(), child, lan_neighbor)),
                None => segments.extend(
                    named_children(child, "locator")
                        .map(|(arg, locator)| segment(Some(protection), arg, locator, lan_neighbor)),
                ),
            }
        }
    }
    segments
}

/// The `srv6-adjacency-segment` of each `lan-neighbor <system-id>` under `level`.
pub(super) fn parse_lan_neighbor_segments(level: &ConfigLine) -> Vec<IsisSrv6AdjacencySegment> {
    let mut segments = Vec::new();
    for (arg, neighbor) in named_children(level, "lan-neighbor") {
        let (id, rest) = split_name(arg);
        segments.extend(parse_adjacency_segments(neighbor, Some(id)));
        if let Some(inline) = rest.strip_prefix("srv6-adjacency-segment") {
            segments.extend(inline_segment(inline.trim(), neighbor, Some(id)));
        }
    }
    segments
}

/// `inline` is `<protected|unprotected> locator <name> end-x-sid <sid> flavor <flavor>`.
fn inline_segment(inline: &str, node: &ConfigLine, lan_neighbor: Option<&str>) -> Option<IsisSrv6AdjacencySegment> {
    let (protection, rest) = split_name(inline);
    let locator = rest.strip_prefix("locator")?;
    Some(segment(Some(protection), locator.trim(), node, lan_neighbor))
}

/// `locator_arg` is the text after `locator`: the name, then any collapsed `end-x-sid` and flavors.
fn segment(
    protection: Option<&str>,
    locator_arg: &str,
    node: &ConfigLine,
    lan_neighbor: Option<&str>,
) -> IsisSrv6AdjacencySegment {
    let (name, inline) = split_name(locator_arg);
    let (end_x_sid, mut flavors) = sid_with_flavors(inline, node, "end-x-sid");
    flavors.extend(flavors_of(node));
    let words: Vec<&str> = inline.split_whitespace().collect();
    let micro = active_children(node)
        .find(|c| c.text == "micro-adjacency-sid" || c.arg("micro-adjacency-sid").is_some())
        .map(|c| c.arg("micro-adjacency-sid").map(str::to_string));
    let micro_inline = words.iter().position(|w| *w == "micro-adjacency-sid");
    let (micro_adjacency_sid, micro_adjacency_sid_value) = match (micro, micro_inline) {
        (Some(value), _) => (true, value),
        (None, Some(at)) => (
            true,
            words.get(at + 1).filter(|w| **w != "flavor").map(|w| w.to_string()),
        ),
        (None, None) => (false, None),
    };
    IsisSrv6AdjacencySegment {
        protection: protection.map(str::to_string),
        locator: name.to_string(),
        end_x_sid,
        flavors,
        micro_adjacency_sid,
        micro_adjacency_sid_value,
        lan_neighbor: lan_neighbor.map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use crate::protocols::isis::isis;
    use asterust::config::juniper;

    // Device R0 `show protocols` (IS-IS part) of "Example: Configuring SRv6 Network Programming in IS-IS Networks"
    // (Segment Routing User Guide), shortened; see the README "Sources".
    const DOC_SRV6: &str = r#"
protocols {
    isis {
        interface xe-0/0/0:0.0 {
            level 2 {
                srv6-adjacency-segment {
                    protected {
                        locator myloc {
                            end-x-sid 2001:db8:0:a0:1a01:: {
                                flavor psp;
                            }
                        }
                    }
                }
            }
            node-link-protection;
            point-to-point;
        }
        interface xe-0/0/0:2.0 {
            level 2 {
                srv6-adjacency-segment {
                    protected {
                        locator myloc {
                            end-x-sid 2001:db8:0:a0:1a04:: {
                                flavor psp;
                            }
                        }
                    }
                }
            }
            node-link-protection;
            point-to-point;
        }
        interface lo0.0 {
            passive;
        }
        source-packet-routing {
            srv6 {
                locator myloc {
                    end-sid 2001:db8:0:a0:d01:: {
                        flavor {
                            usd;
                        }
                    }
                }
            }
        }
        level 1 disable;
    }
}
"#;

    // Not from documentation: written from the CLI Reference, the guide's step-by-step commands and the uSID example at
    // srv6.md. See the README "Sources".
    const EXTRA: &str = r#"
protocols {
    isis {
        interface xe-0/0/0:0.0 {
            level 2 {
                srv6-adjacency-segment {
                    protected {
                        locator myloc {
                            end-x-sid 2001:db8:0:a0:1a01:: {
                                flavor psp;
                            }
                        }
                        locator myloc1 {
                            end-x-sid 2001:db8:0:a10:1a01:: {
                                flavor {
                                    usd;
                                    usp;
                                }
                            }
                        }
                    }
                }
                lan-neighbor 0100.0404.0404 {
                    srv6-adjacency-segment {
                        unprotected {
                            locator myloc {
                                end-x-sid 2001:db8:0:a0:1a04:: {
                                    flavor usd;
                                }
                            }
                        }
                    }
                }
            }
        }
        interface et-0/0/0.0 {
            level 2 {
                srv6-adjacency-segment {
                    unprotected {
                        locator LOC0 {
                            micro-adjacency-sid;
                        }
                        locator LOC1 {
                            micro-adjacency-sid 0xfc10;
                        }
                    }
                }
            }
        }
        source-packet-routing {
            srv6 {
                transit-srh-insert;
                locator LOC0 {
                    micro-node-sid;
                }
                locator LOC128 {
                    algorithm 128;
                    micro-node-sid;
                }
                locator DYN {
                    dynamic-end-sid;
                }
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_guide_node_segment() {
        let spr = isis(&juniper::parse_braces(DOC_SRV6))
            .unwrap()
            .source_packet_routing
            .unwrap();
        let srv6 = spr.srv6.unwrap();
        assert!(!srv6.transit_srh_insert);
        assert_eq!(srv6.locators.len(), 1);
        let locator = &srv6.locators[0];
        assert_eq!(locator.name, "myloc");
        assert_eq!(locator.end_sid.as_deref(), Some("2001:db8:0:a0:d01::"));
        assert_eq!(locator.end_sid_flavors, ["usd"]);
        assert!(!locator.micro_node_sid);
    }

    #[test]
    fn reads_the_guide_adjacency_segments() {
        let isis = isis(&juniper::parse_braces(DOC_SRV6)).unwrap();
        let first = &isis.interfaces[0].levels[0].srv6_adjacency_segments;
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].protection.as_deref(), Some("protected"));
        assert_eq!(first[0].locator, "myloc");
        assert_eq!(first[0].end_x_sid.as_deref(), Some("2001:db8:0:a0:1a01::"));
        assert_eq!(first[0].flavors, ["psp"]);
        assert_eq!(first[0].lan_neighbor, None);

        let second = &isis.interfaces[1].levels[0].srv6_adjacency_segments;
        assert_eq!(second[0].end_x_sid.as_deref(), Some("2001:db8:0:a0:1a04::"));
        assert!(isis.interfaces[2].levels.is_empty());
    }

    #[test]
    fn reads_several_locators_and_lan_neighbors() {
        let isis = isis(&juniper::parse_braces(EXTRA)).unwrap();
        let segments = &isis.interfaces[0].levels[0].srv6_adjacency_segments;
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[1].locator, "myloc1");
        assert_eq!(segments[1].flavors, ["usd", "usp"]);
        assert_eq!(segments[2].protection.as_deref(), Some("unprotected"));
        assert_eq!(segments[2].lan_neighbor.as_deref(), Some("0100.0404.0404"));
        assert_eq!(segments[2].flavors, ["usd"]);
    }

    #[test]
    fn reads_micro_adjacency_sids() {
        let isis = isis(&juniper::parse_braces(EXTRA)).unwrap();
        let segments = &isis.interfaces[1].levels[0].srv6_adjacency_segments;
        assert!(segments[0].micro_adjacency_sid);
        assert_eq!(segments[0].micro_adjacency_sid_value, None);
        assert_eq!(segments[1].micro_adjacency_sid_value.as_deref(), Some("0xfc10"));
        assert_eq!(segments[1].end_x_sid, None);
    }

    #[test]
    fn reads_micro_node_and_dynamic_locators() {
        let spr = isis(&juniper::parse_braces(EXTRA))
            .unwrap()
            .source_packet_routing
            .unwrap();
        let srv6 = spr.srv6.unwrap();
        assert!(srv6.transit_srh_insert);
        assert!(srv6.locators[0].micro_node_sid);
        assert_eq!(srv6.locators[0].algorithm, None);
        assert_eq!(srv6.locators[1].algorithm.as_deref(), Some("128"));
        assert!(srv6.locators[2].dynamic_end_sid);
        assert!(!srv6.locators[2].micro_node_sid);
    }

    #[test]
    fn reads_the_collapsed_forms() {
        let config = juniper::parse_braces(
            "protocols {\n    isis {\n        interface ge-0/0/0.0 {\n            level 2 {\n                srv6-adjacency-segment protected locator myloc end-x-sid 2001:db8::1a01:: flavor psp;\n                lan-neighbor 0100.0404.0404 srv6-adjacency-segment unprotected locator myloc end-x-sid 2001:db8::1a04:: flavor usd;\n            }\n        }\n        source-packet-routing {\n            srv6 {\n                locator myloc end-sid 2001:db8::d01:: flavor usd;\n            }\n        }\n    }\n}\n",
        );
        let isis = isis(&config).unwrap();
        let segments = &isis.interfaces[0].levels[0].srv6_adjacency_segments;
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].end_x_sid.as_deref(), Some("2001:db8::1a01::"));
        assert_eq!(segments[0].flavors, ["psp"]);
        assert_eq!(segments[1].lan_neighbor.as_deref(), Some("0100.0404.0404"));
        assert_eq!(segments[1].flavors, ["usd"]);

        let locator = &isis.source_packet_routing.unwrap().srv6.unwrap().locators[0];
        assert_eq!(locator.end_sid.as_deref(), Some("2001:db8::d01::"));
        assert_eq!(locator.end_sid_flavors, ["usd"]);
    }
}
