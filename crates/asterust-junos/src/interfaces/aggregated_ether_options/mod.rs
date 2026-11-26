//! `aggregated-ether-options`
//! `interfaces { ae<n> { aggregated-ether-options { ... } } }`.
//!
//! # See also
//!
//! - [aggregated-ether-options](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/aggregated-ether-options-edit-interfaces.html)
//! - [link-speed (Aggregated Ethernet)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/link-speed-edit-interfaces-ae.html)
//! - [minimum-links](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/minimum-links-edit-interfaces.html)

mod bfd_liveness_detection;
mod lacp;

pub use bfd_liveness_detection::BfdLivenessDetection;
pub use lacp::Lacp;

use crate::{arg_of, child_block, toggle};
use asterust::config::ConfigLine;
use bfd_liveness_detection::parse_bfd_liveness_detection;
use lacp::parse_lacp;

/// The `aggregated-ether-options` of an aggregated Ethernet interface.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AggregatedEtherOptions {
    pub lacp: Option<Lacp>,
    pub bfd_liveness_detection: Option<BfdLivenessDetection>,
    /// e.g. `"10g"` or `"mixed"`.
    pub link_speed: Option<String>,
    pub minimum_links: Option<String>,
    /// `Some(true)` for `flow-control`, `Some(false)` for `no-flow-control`.
    pub flow_control: Option<bool>,
    /// `Some(true)` for `loopback`, `Some(false)` for `no-loopback`.
    pub loopback: Option<bool>,
    /// `Some(true)` for `source-filtering`, `Some(false)` for `no-source-filtering`.
    pub source_filtering: Option<bool>,
}

pub(super) fn parse_aggregated_ether_options(options: &ConfigLine) -> AggregatedEtherOptions {
    AggregatedEtherOptions {
        lacp: child_block(options, "lacp").map(parse_lacp),
        bfd_liveness_detection: child_block(options, "bfd-liveness-detection").map(parse_bfd_liveness_detection),
        link_speed: arg_of(options, "link-speed").map(str::to_string),
        minimum_links: arg_of(options, "minimum-links").map(str::to_string),
        flow_control: toggle(options, "flow-control"),
        loopback: toggle(options, "loopback"),
        source_filtering: toggle(options, "source-filtering"),
    }
}

#[cfg(test)]
mod tests {
    use crate::interfaces::interfaces;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_aggregated_ether_options_and_lacp() {
        let ifaces = interfaces(&juniper::parse_braces(
            r#"
interfaces {
    ae0 {
        aggregated-ether-options {
            link-speed 10g;
            minimum-links 2;
            no-flow-control;
            lacp {
                active;
                periodic fast;
                system-priority 100;
                system-id 00:11:22:33:44:55;
                admin-key 7;
                force-up;
                fast-failover;
                accept-data;
            }
        }
    }
}
"#,
        ));
        let options = ifaces[0].aggregated_ether_options.as_ref().unwrap();
        assert_eq!(options.link_speed.as_deref(), Some("10g"));
        assert_eq!(options.minimum_links.as_deref(), Some("2"));
        assert_eq!(options.flow_control, Some(false));
        assert_eq!(options.loopback, None);
        assert_eq!(options.source_filtering, None);

        let lacp = options.lacp.as_ref().unwrap();
        assert_eq!(lacp.mode.as_deref(), Some("active"));
        assert_eq!(lacp.periodic.as_deref(), Some("fast"));
        assert_eq!(lacp.system_priority.as_deref(), Some("100"));
        assert_eq!(lacp.system_id.as_deref(), Some("00:11:22:33:44:55"));
        assert_eq!(lacp.admin_key.as_deref(), Some("7"));
        assert!(lacp.force_up);
        assert!(lacp.fast_failover);
        assert!(lacp.accept_data);
    }

    #[test]
    fn bare_lacp_has_no_settings() {
        let ifaces = interfaces(&juniper::parse_braces(
            "interfaces {\n    ae1 {\n        aggregated-ether-options {\n            lacp {\n                passive;\n            }\n        }\n    }\n}\n",
        ));
        let lacp = ifaces[0]
            .aggregated_ether_options
            .as_ref()
            .unwrap()
            .lacp
            .as_ref()
            .unwrap();
        assert_eq!(lacp.mode.as_deref(), Some("passive"));
        assert_eq!(lacp.periodic, None);
        assert!(!lacp.force_up);
    }

    #[test]
    fn aggregated_ether_options_are_optional() {
        let ifaces = interfaces(&juniper::parse_braces(
            "interfaces {\n    ae0 {\n        disable;\n    }\n}\n",
        ));
        assert!(ifaces[0].aggregated_ether_options.is_none());
    }
}
