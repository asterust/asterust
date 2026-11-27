//! `traffic-engineering`
//! `protocols { isis { traffic-engineering { ... } } }`.
//!
//! # See also
//!
//! - [traffic-engineering (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/traffic-engineering-edit-protocols-isis.html)
//! - [family (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/family-edit-protocols-isis.html)
//! - [advertisement](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/protocols-isis-traffic-engineering-advertisement.html)

use crate::{named_children, split_name, statement_words};
use asterust::config::ConfigLine;

/// The IS-IS `traffic-engineering` statement. A bare `traffic-engineering;` leaves every field unset.
/// `tunnel-source-protocol` and the application-specific advertisement settings are not extracted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct IsisTrafficEngineering {
    pub disable: bool,
    pub credibility_protocol_preference: bool,
    pub ignore_lsp_metrics: bool,
    pub ipv6: bool,
    pub l3_unicast_topology: bool,
    /// An `advertisement` block is configured.
    pub advertisement: bool,
    /// `advertisement { always; }`.
    pub advertisement_always: bool,
    /// A `multipath` block is configured.
    pub multipath: bool,
    /// `multipath { lsp-equal-cost; }`.
    pub multipath_lsp_equal_cost: bool,
    pub families: Vec<IsisTeFamily>,
}

/// A `family <inet | inet-mpls | inet6 | inet6-mpls>` of `traffic-engineering`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct IsisTeFamily {
    pub name: String,
    pub shortcuts: bool,
    /// `shortcuts { multicast-rpf-routes; }`, valid for `inet`.
    pub multicast_rpf_routes: bool,
}

pub(super) fn parse_traffic_engineering(isis: &ConfigLine) -> Option<IsisTrafficEngineering> {
    let words = statement_words(isis, "traffic-engineering")?;
    let te = crate::child_block(isis, "traffic-engineering");
    let advertisement = te.and_then(|te| statement_words(te, "advertisement"));
    let multipath = te.and_then(|te| statement_words(te, "multipath"));
    Some(IsisTrafficEngineering {
        disable: words.contains(&"disable"),
        credibility_protocol_preference: words.contains(&"credibility-protocol-preference"),
        ignore_lsp_metrics: words.contains(&"ignore-lsp-metrics"),
        ipv6: words.contains(&"ipv6"),
        l3_unicast_topology: words.contains(&"l3-unicast-topology"),
        advertisement: advertisement.is_some(),
        advertisement_always: advertisement.is_some_and(|words| words.contains(&"always")),
        multipath: multipath.is_some(),
        multipath_lsp_equal_cost: multipath.is_some_and(|words| words.contains(&"lsp-equal-cost")),
        families: te.map(parse_families).unwrap_or_default(),
    })
}

fn parse_families(te: &ConfigLine) -> Vec<IsisTeFamily> {
    named_children(te, "family")
        .map(|(arg, family)| {
            let (name, rest) = split_name(arg);
            let inline = rest.split_whitespace().collect::<Vec<_>>();
            let shortcuts = statement_words(family, "shortcuts");
            let has =
                |flag: &str| inline.contains(&flag) || shortcuts.as_ref().is_some_and(|words| words.contains(&flag));
            IsisTeFamily {
                name: name.to_string(),
                shortcuts: has("shortcuts") || shortcuts.is_some(),
                multicast_rpf_routes: has("multicast-rpf-routes"),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::protocols::isis::isis;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    isis {
        traffic-engineering {
            credibility-protocol-preference;
            ignore-lsp-metrics;
            ipv6;
            l3-unicast-topology;
            advertisement {
                always;
            }
            multipath {
                lsp-equal-cost;
            }
            family inet {
                shortcuts {
                    multicast-rpf-routes;
                }
            }
            family inet6 {
                shortcuts;
            }
            family inet6-mpls;
        }
    }
}
"#;

    #[test]
    fn reads_the_flags_and_blocks() {
        let te = isis(&juniper::parse_braces(CONFIG))
            .unwrap()
            .traffic_engineering
            .unwrap();
        assert!(te.credibility_protocol_preference);
        assert!(te.ignore_lsp_metrics);
        assert!(te.ipv6);
        assert!(te.l3_unicast_topology);
        assert!(te.advertisement);
        assert!(te.advertisement_always);
        assert!(te.multipath);
        assert!(te.multipath_lsp_equal_cost);
        assert!(!te.disable);
    }

    #[test]
    fn reads_the_families() {
        let te = isis(&juniper::parse_braces(CONFIG))
            .unwrap()
            .traffic_engineering
            .unwrap();
        let families: Vec<_> = te
            .families
            .iter()
            .map(|family| (family.name.as_str(), family.shortcuts, family.multicast_rpf_routes))
            .collect();
        assert_eq!(
            families,
            [
                ("inet", true, true),
                ("inet6", true, false),
                ("inet6-mpls", false, false)
            ]
        );
    }

    #[test]
    fn reads_the_bare_form_and_absence() {
        let bare = juniper::parse_braces("protocols {\n    isis {\n        traffic-engineering;\n    }\n}\n");
        let te = isis(&bare).unwrap().traffic_engineering.unwrap();
        assert!(te.families.is_empty());
        assert!(!te.multipath);

        let none = juniper::parse_braces("protocols {\n    isis {\n        interface ge-0/0/0.0;\n    }\n}\n");
        assert_eq!(isis(&none).unwrap().traffic_engineering, None);
    }
}
