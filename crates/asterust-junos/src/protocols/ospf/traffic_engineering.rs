//! `traffic-engineering`
//! `protocols { ospf { traffic-engineering { ... } } }`.
//!
//! # See also
//!
//! - [traffic-engineering (OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/traffic-engineering-edit-protocols-ospf.html)
//! - [shortcuts (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/shortcuts-edit-protocols-ospf.html)
//! - [ignore-lsp-metrics](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/ignore-lsp-metrics-edit-protocols-ospf-mp.html)

use crate::{child_block, statement_words};
use asterust::config::ConfigLine;

/// The OSPF `traffic-engineering` statement. A bare `traffic-engineering;` leaves every flag unset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfTrafficEngineering {
    pub advertise_unnumbered_interfaces: bool,
    pub credibility_protocol_preference: bool,
    pub multicast_rpf_routes: bool,
    pub no_topology: bool,
    pub l3_unicast_topology: bool,
    /// A `shortcuts` block is configured.
    pub shortcuts: bool,
    /// `shortcuts { lsp-metric-into-summary; }`.
    pub lsp_metric_into_summary: bool,
    /// Under `shortcuts`, or directly under `traffic-engineering` as the `ospf` page shows it.
    pub ignore_lsp_metrics: bool,
}

pub(super) fn parse_traffic_engineering(ospf: &ConfigLine) -> Option<OspfTrafficEngineering> {
    let words = statement_words(ospf, "traffic-engineering")?;
    let shortcuts = child_block(ospf, "traffic-engineering").and_then(|te| statement_words(te, "shortcuts"));
    let in_shortcuts = |flag: &str| shortcuts.as_ref().is_some_and(|words| words.contains(&flag));
    Some(OspfTrafficEngineering {
        advertise_unnumbered_interfaces: words.contains(&"advertise-unnumbered-interfaces"),
        credibility_protocol_preference: words.contains(&"credibility-protocol-preference"),
        multicast_rpf_routes: words.contains(&"multicast-rpf-routes"),
        no_topology: words.contains(&"no-topology"),
        l3_unicast_topology: words.contains(&"l3-unicast-topology"),
        shortcuts: shortcuts.is_some(),
        lsp_metric_into_summary: in_shortcuts("lsp-metric-into-summary"),
        ignore_lsp_metrics: words.contains(&"ignore-lsp-metrics") || in_shortcuts("ignore-lsp-metrics"),
    })
}

#[cfg(test)]
mod tests {
    use crate::protocols::ospf::ospf;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    ospf {
        traffic-engineering {
            advertise-unnumbered-interfaces;
            credibility-protocol-preference;
            no-topology;
            shortcuts {
                lsp-metric-into-summary;
                ignore-lsp-metrics;
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_flags_and_shortcuts() {
        let te = ospf(&juniper::parse_braces(CONFIG))
            .unwrap()
            .traffic_engineering
            .unwrap();
        assert!(te.advertise_unnumbered_interfaces);
        assert!(te.credibility_protocol_preference);
        assert!(te.no_topology);
        assert!(te.shortcuts);
        assert!(te.lsp_metric_into_summary);
        assert!(te.ignore_lsp_metrics);
        assert!(!te.multicast_rpf_routes);
    }

    #[test]
    fn reads_ignore_lsp_metrics_directly_under_the_statement() {
        let config = juniper::parse_braces("protocols {\n    ospf {\n        traffic-engineering {\n            ignore-lsp-metrics;\n        }\n    }\n}\n");
        let te = ospf(&config).unwrap().traffic_engineering.unwrap();
        assert!(te.ignore_lsp_metrics);
        assert!(!te.shortcuts);
    }
}
