//! `source-packet-routing srv6`
//! `routing-options { source-packet-routing { srv6 { block ...; locator ...; } } }`.
//!
//! # See also
//!
//! - [srv6 (Routing Options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/srv6-edit-routing-options-source-packet-routing.html)
//! - [locator](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/locator-edit-routing-options-source-packet-routing-srv6.html)
//! - [micro-sid](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/routing-options-source-packet-routing-srv6-locator-micro-sid.html)
//! - [block](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/routing-options-source-packet-routing-srv6-block.html)
//! - [Overview of SRv6 Network Programming in IS-IS Networks](https://networkingdocs.hpe.com/documentation/us/en/software/junos/is-is/segment-routing/topics/concept/isis-srv6-network-programming.html)
//! - [Example: Configuring SRv6 Network Programming in IS-IS Networks](https://networkingdocs.hpe.com/documentation/us/en/software/junos/segment-routing/topics/example/isis-configuring-srv6-network-programming.html)

use crate::{active_children, child_block, first_word, has_leaf, list_values, named_children, split_name, value_after};
use asterust::config::ConfigLine;

/// `routing-options source-packet-routing srv6`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Srv6 {
    pub no_reduced_srh: bool,
    pub transit_srh_insert: bool,
    pub oam: bool,
    pub blocks: Vec<Srv6Block>,
    pub locators: Vec<Srv6Locator>,
}

/// A micro-SID `block <name> <prefix>`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Srv6Block {
    pub name: String,
    pub prefix: Option<String>,
    /// `global-micro-sid { maximum-sids ...; }`.
    pub global_maximum_sids: Option<String>,
    /// `global-micro-sid { maximum-static-sids ...; }`.
    pub global_maximum_static_sids: Option<String>,
    /// `local-micro-sid { maximum-static-sids ...; }`.
    pub local_maximum_static_sids: Option<String>,
}

/// A `locator <name> <prefix>`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Srv6Locator {
    pub name: String,
    pub prefix: Option<String>,
    /// The flexible algorithm the locator is bound to.
    pub algorithm: Option<String>,
    pub anycast: bool,
    pub end_sid: Option<String>,
    pub dynamic_end_sid: bool,
    /// Flavors of the `end-sid`: `psp`, `usp` and `usd`.
    pub end_sid_flavors: Vec<String>,
    pub micro_sid: Option<Srv6MicroSid>,
}

/// The `micro-sid` options of a [`Srv6Locator`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Srv6MicroSid {
    pub anycast: bool,
    pub block_name: Option<String>,
    /// `none`, `psp`, `usp` and `usd`.
    pub flavors: Vec<String>,
}

pub(super) fn parse_srv6(root: &ConfigLine) -> Option<Srv6> {
    let srv6 = child_block(child_block(root, "source-packet-routing")?, "srv6")?;
    Some(Srv6 {
        no_reduced_srh: has_leaf(srv6, "no-reduced-srh"),
        transit_srh_insert: has_leaf(srv6, "transit-srh-insert"),
        oam: has_leaf(srv6, "oam"),
        blocks: named_children(srv6, "block")
            .map(|(arg, block)| parse_block(arg, block))
            .collect(),
        locators: named_children(srv6, "locator")
            .map(|(arg, locator)| parse_locator(arg, locator))
            .collect(),
    })
}

fn parse_block(arg: &str, block: &ConfigLine) -> Srv6Block {
    let (name, rest) = split_name(arg);
    let setting = |keyword: &str, option: &str| {
        let node = active_children(block).find(|c| c.text == keyword)?;
        let words: Vec<&str> = active_children(node).flat_map(|c| c.text.split_whitespace()).collect();
        value_after(&words, option)
    };
    Srv6Block {
        name: name.to_string(),
        prefix: first_word(rest).map(str::to_string),
        global_maximum_sids: setting("global-micro-sid", "maximum-sids"),
        global_maximum_static_sids: setting("global-micro-sid", "maximum-static-sids"),
        local_maximum_static_sids: setting("local-micro-sid", "maximum-static-sids"),
    }
}

fn parse_locator(arg: &str, locator: &ConfigLine) -> Srv6Locator {
    let (name, rest) = split_name(arg);
    let (end_sid, end_sid_flavors) = sid_with_flavors(rest, locator, "end-sid");
    Srv6Locator {
        name: name.to_string(),
        prefix: first_word(rest).map(str::to_string),
        algorithm: active_children(locator)
            .find_map(|c| c.arg("algorithm"))
            .and_then(first_word)
            .map(str::to_string),
        anycast: has_leaf(locator, "anycast"),
        end_sid,
        dynamic_end_sid: has_leaf(locator, "dynamic-end-sid"),
        end_sid_flavors,
        micro_sid: active_children(locator)
            .find(|c| c.text == "micro-sid" || c.arg("micro-sid").is_some())
            .map(parse_micro_sid),
    }
}

fn parse_micro_sid(node: &ConfigLine) -> Srv6MicroSid {
    let inline = node.arg("micro-sid").unwrap_or_default();
    let mut flavors = flavors_of(node);
    flavors.extend(inline_flavors(inline));
    let block_name = active_children(node).find_map(|c| c.arg("block-name")).or_else(|| {
        let words: Vec<&str> = inline.split_whitespace().collect();
        words
            .iter()
            .position(|w| *w == "block-name")
            .and_then(|i| words.get(i + 1).copied())
    });
    Srv6MicroSid {
        anycast: has_leaf(node, "anycast") || inline.split_whitespace().any(|w| w == "anycast"),
        block_name: block_name.map(str::to_string),
        flavors,
    }
}

/// Flavors written under a `flavor { ...; }` block or as `flavor <name>;` among the children of `node`.
pub(crate) fn flavors_of(node: &ConfigLine) -> Vec<String> {
    let mut flavors = Vec::new();
    for child in active_children(node) {
        if child.text == "flavor" {
            flavors.extend(active_children(child).map(|flavor| flavor.text.clone()));
        } else if let Some(arg) = child.arg("flavor") {
            flavors.extend(list_values(arg));
        }
    }
    flavors
}

/// The SID of `<keyword> <sid>` and its flavors. The statement is a child of `node`, or in the collapsed one-line
/// `inline` text.
pub(crate) fn sid_with_flavors(inline: &str, node: &ConfigLine, keyword: &str) -> (Option<String>, Vec<String>) {
    if let Some((arg, statement)) = active_children(node).find_map(|c| c.arg(keyword).map(|arg| (arg, c))) {
        let (sid, rest) = split_name(arg);
        let mut flavors = flavors_of(statement);
        flavors.extend(inline_flavors(rest));
        return (Some(sid.to_string()), flavors);
    }
    let words: Vec<&str> = inline.split_whitespace().collect();
    match words.iter().position(|word| *word == keyword) {
        Some(at) => (
            words.get(at + 1).map(|sid| sid.to_string()),
            inline_flavors(&words[(at + 2).min(words.len())..].join(" ")),
        ),
        None => (None, Vec::new()),
    }
}

/// Flavors in a collapsed one-line statement such as `end-sid 2001:db8::1 flavor usd`.
fn inline_flavors(inline: &str) -> Vec<String> {
    inline
        .split_once("flavor")
        .map(|(_, flavors)| list_values(flavors))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use crate::routing_options::routing_options;
    use asterust::config::juniper;

    // Device R0 `show routing-options` of "Example: Configuring SRv6 Network Programming in IS-IS Networks"
    // (Segment Routing User Guide); see the README "Sources".
    const DOC_SRV6: &str = r#"
routing-options {
    source-packet-routing {
        srv6 {
            locator myloc 2001:db8:0:a0::/64;
        }
    }
    forwarding-table {
        export pplb;
    }
    router-id 172.16.255.10;
    autonomous-system 65550;
}
"#;

    // Not from documentation: written from the CLI Reference (`block`, `locator`, `micro-sid`, `srv6`) and the uSID
    // example at srv6.md. See the README "Sources".
    const MICRO_SID: &str = r#"
routing-options {
    source-packet-routing {
        srv6 {
            no-reduced-srh;
            transit-srh-insert;
            oam;
            block BLK0 5f00::/32 {
                global-micro-sid {
                    maximum-sids 40000;
                    maximum-static-sids 100;
                }
                local-micro-sid {
                    maximum-static-sids 2000;
                }
            }
            locator LOC0 5f00:0:1::/48 {
                micro-sid {
                    block-name BLK0;
                    flavor {
                        psp;
                        usd;
                    }
                }
            }
            locator LOC128 5f00:a1:1::/48 {
                algorithm 128;
                micro-sid {
                    anycast;
                    block-name BLK128;
                }
            }
            locator CLASSIC 2001:db8:0:a1::/64 {
                end-sid 2001:db8:0:a1:d11:: {
                    flavor {
                        usd;
                    }
                }
            }
            locator DYNAMIC 2001:db8:0:a2::/64 {
                dynamic-end-sid;
                anycast;
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_guide_locator() {
        let options = routing_options(&juniper::parse_braces(DOC_SRV6)).unwrap();
        let srv6 = options.srv6.unwrap();
        assert_eq!(srv6.locators.len(), 1);
        assert_eq!(srv6.locators[0].name, "myloc");
        assert_eq!(srv6.locators[0].prefix.as_deref(), Some("2001:db8:0:a0::/64"));
        assert!(srv6.blocks.is_empty());
        assert!(!srv6.no_reduced_srh);
        assert_eq!(options.router_id.as_deref(), Some("172.16.255.10"));
    }

    #[test]
    fn reads_the_options_and_blocks() {
        let srv6 = routing_options(&juniper::parse_braces(MICRO_SID))
            .unwrap()
            .srv6
            .unwrap();
        assert!(srv6.no_reduced_srh);
        assert!(srv6.transit_srh_insert);
        assert!(srv6.oam);
        let block = &srv6.blocks[0];
        assert_eq!(block.name, "BLK0");
        assert_eq!(block.prefix.as_deref(), Some("5f00::/32"));
        assert_eq!(block.global_maximum_sids.as_deref(), Some("40000"));
        assert_eq!(block.global_maximum_static_sids.as_deref(), Some("100"));
        assert_eq!(block.local_maximum_static_sids.as_deref(), Some("2000"));
    }

    #[test]
    fn reads_micro_sid_locators() {
        let srv6 = routing_options(&juniper::parse_braces(MICRO_SID))
            .unwrap()
            .srv6
            .unwrap();
        assert_eq!(srv6.locators.len(), 4);

        let node = &srv6.locators[0];
        assert_eq!(node.prefix.as_deref(), Some("5f00:0:1::/48"));
        let micro = node.micro_sid.as_ref().unwrap();
        assert_eq!(micro.block_name.as_deref(), Some("BLK0"));
        assert_eq!(micro.flavors, ["psp", "usd"]);
        assert!(!micro.anycast);

        let flex = &srv6.locators[1];
        assert_eq!(flex.algorithm.as_deref(), Some("128"));
        let micro = flex.micro_sid.as_ref().unwrap();
        assert!(micro.anycast);
        assert_eq!(micro.block_name.as_deref(), Some("BLK128"));
        assert!(micro.flavors.is_empty());
    }

    #[test]
    fn reads_classic_locators() {
        let srv6 = routing_options(&juniper::parse_braces(MICRO_SID))
            .unwrap()
            .srv6
            .unwrap();
        let classic = &srv6.locators[2];
        assert_eq!(classic.end_sid.as_deref(), Some("2001:db8:0:a1:d11::"));
        assert_eq!(classic.end_sid_flavors, ["usd"]);
        assert_eq!(classic.micro_sid, None);

        let dynamic = &srv6.locators[3];
        assert!(dynamic.dynamic_end_sid);
        assert!(dynamic.anycast);
        assert_eq!(dynamic.end_sid, None);
    }

    #[test]
    fn reads_the_collapsed_forms() {
        let config = juniper::parse_braces(
            "routing-options {\n    source-packet-routing {\n        srv6 {\n            locator L 2001:db8::/64 {\n                end-sid 2001:db8::d1:: flavor usd;\n                micro-sid block-name B flavor psp;\n            }\n        }\n    }\n}\n",
        );
        let srv6 = routing_options(&config).unwrap().srv6.unwrap();
        let locator = &srv6.locators[0];
        assert_eq!(locator.end_sid.as_deref(), Some("2001:db8::d1::"));
        assert_eq!(locator.end_sid_flavors, ["usd"]);
        let micro = locator.micro_sid.as_ref().unwrap();
        assert_eq!(micro.block_name.as_deref(), Some("B"));
        assert_eq!(micro.flavors, ["psp"]);
    }

    #[test]
    fn no_srv6_is_none() {
        let config = juniper::parse_braces("routing-options {\n    router-id 192.0.2.1;\n}\n");
        assert_eq!(routing_options(&config).unwrap().srv6, None);
    }
}
