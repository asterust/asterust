//! `interfaces { <name> { ether-options { ... } } }`, also written `gigether-options` or `fastether-options`.

mod ieee_802_3ad;

pub use ieee_802_3ad::Ieee8023ad;

use crate::{arg_of, child_block, statement_words, toggle, value_after};
use asterust::config::ConfigLine;
use ieee_802_3ad::parse_ieee_802_3ad;

/// The Ethernet options of a physical interface.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct EtherOptions {
    /// `Some(true)` for `auto-negotiation`, `Some(false)` for `no-auto-negotiation`.
    pub auto_negotiation: Option<bool>,
    /// `local-interface-online` or `local-interface-offline`.
    pub remote_fault: Option<String>,
    /// `Some(true)` for `flow-control`, `Some(false)` for `no-flow-control`.
    pub flow_control: Option<bool>,
    /// `Some(true)` for `loopback`, `Some(false)` for `no-loopback`.
    pub loopback: Option<bool>,
    /// `Some(true)` for `source-filtering`, `Some(false)` for `no-source-filtering`.
    pub source_filtering: Option<bool>,
    pub speed: Option<String>,
    /// `full-duplex`, `half-duplex` or `automatic`.
    pub link_mode: Option<String>,
    /// e.g. `"fec91"`, `"fec74"`, `"fec108"` or `"none"`.
    pub fec: Option<String>,
    /// The bundle this interface is a member of; the statement is `802.3ad`.
    pub ieee_802_3ad: Option<Ieee8023ad>,
}

pub(super) fn parse_ether_options(interface: &ConfigLine) -> Option<EtherOptions> {
    let options = ["ether-options", "gigether-options", "fastether-options"]
        .iter()
        .find_map(|name| child_block(interface, name))?;
    let negotiation =
        statement_words(options, "auto-negotiation").or_else(|| statement_words(options, "no-auto-negotiation"));
    Some(EtherOptions {
        auto_negotiation: toggle(options, "auto-negotiation"),
        remote_fault: negotiation.and_then(|words| value_after(&words, "remote-fault")),
        flow_control: toggle(options, "flow-control"),
        loopback: toggle(options, "loopback"),
        source_filtering: toggle(options, "source-filtering"),
        speed: arg_of(options, "speed").map(str::to_string),
        link_mode: arg_of(options, "link-mode").map(str::to_string),
        fec: arg_of(options, "fec").map(str::to_string),
        ieee_802_3ad: parse_ieee_802_3ad(options),
    })
}

#[cfg(test)]
mod tests {
    use crate::interfaces::interfaces;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
interfaces {
    ge-0/0/0 {
        gigether-options {
            802.3ad ae0;
            no-auto-negotiation;
            no-flow-control;
            loopback;
            source-filtering;
            speed 1g;
        }
    }
    ge-0/0/1 {
        gigether-options {
            802.3ad ae1 {
                primary;
                link-index 2;
                distribution-list DL1;
                lacp {
                    port-priority 64;
                }
            }
            auto-negotiation remote-fault local-interface-offline;
        }
    }
    xe-0/0/0 {
        ether-options {
            802.3ad {
                ae2;
                backup;
            }
            link-mode full-duplex;
            fec fec91;
            no-loopback;
            no-source-filtering;
            flow-control;
        }
    }
}
"#;

    #[test]
    fn extracts_ether_options_and_bundle_membership() {
        let ifaces = interfaces(&juniper::parse_braces(CONFIG));
        let options = ifaces[0].ether_options.as_ref().unwrap();
        assert_eq!(options.auto_negotiation, Some(false));
        assert_eq!(options.remote_fault, None);
        assert_eq!(options.flow_control, Some(false));
        assert_eq!(options.loopback, Some(true));
        assert_eq!(options.source_filtering, Some(true));
        assert_eq!(options.speed.as_deref(), Some("1g"));
        assert_eq!(options.link_mode, None);

        let bundle = options.ieee_802_3ad.as_ref().unwrap();
        assert_eq!(bundle.aggregated_interface.as_deref(), Some("ae0"));
        assert_eq!(bundle.role, None);
    }

    #[test]
    fn extracts_802_3ad_block_and_remote_fault() {
        let ifaces = interfaces(&juniper::parse_braces(CONFIG));
        let options = ifaces[1].ether_options.as_ref().unwrap();
        assert_eq!(options.auto_negotiation, Some(true));
        assert_eq!(options.remote_fault.as_deref(), Some("local-interface-offline"));

        let bundle = options.ieee_802_3ad.as_ref().unwrap();
        assert_eq!(bundle.aggregated_interface.as_deref(), Some("ae1"));
        assert_eq!(bundle.role.as_deref(), Some("primary"));
        assert_eq!(bundle.link_index.as_deref(), Some("2"));
        assert_eq!(bundle.distribution_list.as_deref(), Some("DL1"));
        assert_eq!(bundle.lacp_port_priority.as_deref(), Some("64"));
    }

    #[test]
    fn extracts_ether_options_with_the_bundle_named_inside_the_block() {
        let ifaces = interfaces(&juniper::parse_braces(CONFIG));
        let options = ifaces[2].ether_options.as_ref().unwrap();
        assert_eq!(options.link_mode.as_deref(), Some("full-duplex"));
        assert_eq!(options.fec.as_deref(), Some("fec91"));
        assert_eq!(options.loopback, Some(false));
        assert_eq!(options.source_filtering, Some(false));
        assert_eq!(options.flow_control, Some(true));
        assert_eq!(options.auto_negotiation, None);

        let bundle = options.ieee_802_3ad.as_ref().unwrap();
        assert_eq!(bundle.aggregated_interface.as_deref(), Some("ae2"));
        assert_eq!(bundle.role.as_deref(), Some("backup"));
    }

    #[test]
    fn ether_options_are_optional() {
        let ifaces = interfaces(&juniper::parse_braces(
            "interfaces {\n    ge-0/0/0 {\n        disable;\n    }\n}\n",
        ));
        assert!(ifaces[0].ether_options.is_none());
    }
}
