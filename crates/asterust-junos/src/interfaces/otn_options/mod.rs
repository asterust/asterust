//! Extracts the `otn-options` of an interface.
//!
//! Configuration path: `interfaces { <name> { otn-options { ... } } }`.
//!
//! # See also
//!
//! - [otn-options](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/otn-options-edit-interfaces.html)
//! - [fec](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/fec-edit-interfaces-otn-options.html)
//! - [tti](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/tti-edit-interfaces-otn-options.html)
//! - [bytes (otn-options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/bytes-edit-interfaces-otn-options.html)

mod rate;
mod signal_degrade;

pub use rate::OtnRate;
pub use signal_degrade::SignalDegrade;

use crate::{active_children, arg_of, child_block, has_leaf, statement_words, toggle, value_after};
use asterust::config::ConfigLine;
use rate::parse_rate;
use signal_degrade::parse_signal_degrade;

/// The OTN settings of an interface. `odu-delay-management`, `odu-signal-degrade`,
/// `preemptive-fast-reroute`, `tca`, `trigger` and the loopback statements are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OtnOptions {
    /// e.g. `"gfec"`, `"efec"`, `"ufec"` or `"none"`.
    pub fec: Option<String>,
    /// From `bytes transmit-payload-type`.
    pub transmit_payload_type: Option<String>,
    /// Each `tti` statement as written, e.g. `"otu-sapi \"text\""`.
    pub tti: Vec<String>,
    pub rate: Option<OtnRate>,
    pub signal_degrade: Option<SignalDegrade>,
    pub transport_monitoring: bool,
    /// `Some(true)` for `is-ma`, `Some(false)` for `no-is-ma`.
    pub is_ma: Option<bool>,
    /// `Some(true)` for `laser-enable`, `Some(false)` for `no-laser-enable`.
    pub laser_enable: Option<bool>,
    /// `Some(true)` for `prbs`, `Some(false)` for `no-prbs`.
    pub prbs: Option<bool>,
    pub odu_ttim_action_enable: Option<bool>,
    pub otu_ttim_action_enable: Option<bool>,
}

pub(super) fn parse_otn_options(otn: &ConfigLine) -> OtnOptions {
    OtnOptions {
        fec: arg_of(otn, "fec").map(str::to_string),
        transmit_payload_type: statement_words(otn, "bytes")
            .and_then(|words| value_after(&words, "transmit-payload-type")),
        tti: active_children(otn)
            .filter_map(|c| c.arg("tti"))
            .map(str::to_string)
            .collect(),
        rate: child_block(otn, "rate").map(parse_rate),
        signal_degrade: child_block(otn, "signal-degrade").map(parse_signal_degrade),
        transport_monitoring: has_leaf(otn, "transport-monitoring"),
        is_ma: toggle(otn, "is-ma"),
        laser_enable: toggle(otn, "laser-enable"),
        prbs: toggle(otn, "prbs"),
        odu_ttim_action_enable: toggle(otn, "odu-ttim-action-enable"),
        otu_ttim_action_enable: toggle(otn, "otu-ttim-action-enable"),
    }
}

#[cfg(test)]
mod tests {
    use crate::interfaces::interfaces;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_otn_options() {
        let ifaces = interfaces(&juniper::parse_braces(
            r#"
interfaces {
    et-1/0/0 {
        otn-options {
            fec gfec;
            bytes {
                transmit-payload-type 7;
            }
            tti otu-sapi "LAB1";
            tti odu-dapi "LAB2";
            transport-monitoring;
            laser-enable;
            no-prbs;
            odu-ttim-action-enable;
            rate {
                otu4;
                no-pass-through;
            }
            signal-degrade {
                ber-threshold-signal-degrade 1.0E-5;
                ber-threshold-clear 1.0E-6;
                interval 100;
            }
        }
    }
}
"#,
        ));
        let otn = ifaces[0].otn_options.as_ref().unwrap();
        assert_eq!(otn.fec.as_deref(), Some("gfec"));
        assert_eq!(otn.transmit_payload_type.as_deref(), Some("7"));
        assert_eq!(otn.tti, ["otu-sapi \"LAB1\"", "odu-dapi \"LAB2\""]);
        assert!(otn.transport_monitoring);
        assert_eq!(otn.laser_enable, Some(true));
        assert_eq!(otn.prbs, Some(false));
        assert_eq!(otn.odu_ttim_action_enable, Some(true));
        assert_eq!(otn.otu_ttim_action_enable, None);
        assert_eq!(otn.is_ma, None);

        let rate = otn.rate.as_ref().unwrap();
        assert!(rate.otu4);
        assert!(!rate.oc192);
        assert_eq!(rate.pass_through, Some(false));
        assert_eq!(rate.fixed_stuff_bytes, None);

        let degrade = otn.signal_degrade.as_ref().unwrap();
        assert_eq!(degrade.ber_threshold_signal_degrade.as_deref(), Some("1.0E-5"));
        assert_eq!(degrade.ber_threshold_clear.as_deref(), Some("1.0E-6"));
        assert_eq!(degrade.interval.as_deref(), Some("100"));
    }

    #[test]
    fn bare_otn_options_have_no_settings() {
        let ifaces = interfaces(&juniper::parse_braces(
            "interfaces {\n    et-1/0/0 {\n        otn-options {\n            fec none;\n        }\n    }\n    et-1/0/1 {\n        disable;\n    }\n}\n",
        ));
        let otn = ifaces[0].otn_options.as_ref().unwrap();
        assert_eq!(otn.fec.as_deref(), Some("none"));
        assert!(otn.tti.is_empty());
        assert!(otn.rate.is_none());
        assert!(otn.signal_degrade.is_none());
        assert!(!otn.transport_monitoring);
        assert!(ifaces[1].otn_options.is_none());
    }
}
