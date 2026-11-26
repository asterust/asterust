//! `optics-options`
//! `interfaces { <name> { optics-options { ... } } }`.
//!
//! # See also
//!
//! - [optics-options](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/optics-options-edit-interfaces.html)
//! - [alarm (optics-options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/alarm-edit-interfaces.html)
//! - [warning](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/warning-edit-interfaces-optics-options-qfx-series.html)
//! - [wavelength](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/wavelength-edit-interfaces.html)
//! - [tx-power](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/tx-power-edit-interfaces-optics-options-qfx-series.html)
//! - [fec](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/fec-edit-interfaces-optics-options-qfx-series.html)

use crate::{arg_of, has_leaf, named_children, toggle};
use asterust::config::ConfigLine;

/// The actions of a `low-light-alarm` or `low-light-warning` statement.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct OpticsLowLight {
    /// Take the link down.
    pub link_down: bool,
    pub syslog: bool,
}

/// The optics settings of an interface. `signal-degrade`, `tca`, `loopback`, `lane`,
/// `host-side-fec`, `media-side-fec` and `application` are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OpticsOptions {
    pub low_light_alarm: Option<OpticsLowLight>,
    pub low_light_warning: Option<OpticsLowLight>,
    /// The legacy `alarms;` flag.
    pub alarms: bool,
    /// The ITU grid wavelength in nanometres, e.g. `"1550.12"`.
    pub wavelength: Option<String>,
    /// dBm.
    pub tx_power: Option<String>,
    /// e.g. `"sdfec"` or `"hgfec"`.
    pub fec: Option<String>,
    pub encoding: Option<String>,
    /// e.g. `"16qam"` or `"qpsk"`.
    pub modulation_format: Option<String>,
    pub los_alarm_threshold: Option<String>,
    pub los_warning_threshold: Option<String>,
    pub high_polarization: bool,
    pub high_power_mode: bool,
    pub low_power_mode: bool,
    /// `Some(true)` for `laser-enable`, `Some(false)` for `no-laser-enable`.
    pub laser_enable: Option<bool>,
    /// `Some(true)` for `is-ma`, `Some(false)` for `no-is-ma`.
    pub is_ma: Option<bool>,
}

pub(super) fn parse_optics_options(optics: &ConfigLine) -> OpticsOptions {
    let field = |keyword: &str| arg_of(optics, keyword).map(str::to_string);
    OpticsOptions {
        low_light_alarm: low_light(optics, "alarm", "low-light-alarm"),
        low_light_warning: low_light(optics, "warning", "low-light-warning"),
        alarms: has_leaf(optics, "alarms"),
        wavelength: field("wavelength"),
        tx_power: field("tx-power"),
        fec: field("fec"),
        encoding: field("encoding"),
        modulation_format: field("modulation-format"),
        los_alarm_threshold: field("los-alarm-threshold"),
        los_warning_threshold: field("los-warning-threshold"),
        high_polarization: has_leaf(optics, "high-polarization"),
        high_power_mode: has_leaf(optics, "high-power-mode"),
        low_power_mode: has_leaf(optics, "low-power-mode"),
        laser_enable: toggle(optics, "laser-enable"),
        is_ma: toggle(optics, "is-ma"),
    }
}

/// `<keyword> <name> { link-down; syslog; }`.
fn low_light(optics: &ConfigLine, keyword: &str, name: &str) -> Option<OpticsLowLight> {
    let (_, block) = named_children(optics, keyword).find(|(found, _)| *found == name)?;
    Some(OpticsLowLight {
        link_down: has_leaf(block, "link-down"),
        syslog: has_leaf(block, "syslog"),
    })
}

#[cfg(test)]
mod tests {
    use crate::interfaces::interfaces;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_optics_options() {
        let ifaces = interfaces(&juniper::parse_braces(
            r#"
interfaces {
    et-0/0/0 {
        optics-options {
            alarm low-light-alarm {
                link-down;
            }
            warning low-light-warning {
                syslog;
            }
            wavelength 1550.12;
            tx-power -3;
            fec sdfec;
            modulation-format 16qam;
            encoding differential;
            los-alarm-threshold -20;
            los-warning-threshold -18;
            high-polarization;
            no-laser-enable;
            is-ma;
        }
    }
    xe-0/0/1 {
        optics-options {
            alarms;
            low-power-mode;
        }
    }
}
"#,
        ));
        let optics = ifaces[0].optics_options.as_ref().unwrap();
        let alarm = optics.low_light_alarm.as_ref().unwrap();
        assert!(alarm.link_down);
        assert!(!alarm.syslog);
        let warning = optics.low_light_warning.as_ref().unwrap();
        assert!(!warning.link_down);
        assert!(warning.syslog);
        assert!(!optics.alarms);
        assert_eq!(optics.wavelength.as_deref(), Some("1550.12"));
        assert_eq!(optics.tx_power.as_deref(), Some("-3"));
        assert_eq!(optics.fec.as_deref(), Some("sdfec"));
        assert_eq!(optics.modulation_format.as_deref(), Some("16qam"));
        assert_eq!(optics.encoding.as_deref(), Some("differential"));
        assert_eq!(optics.los_alarm_threshold.as_deref(), Some("-20"));
        assert_eq!(optics.los_warning_threshold.as_deref(), Some("-18"));
        assert!(optics.high_polarization);
        assert!(!optics.high_power_mode);
        assert_eq!(optics.laser_enable, Some(false));
        assert_eq!(optics.is_ma, Some(true));

        let legacy = ifaces[1].optics_options.as_ref().unwrap();
        assert!(legacy.alarms);
        assert!(legacy.low_power_mode);
        assert!(legacy.low_light_alarm.is_none());
        assert_eq!(legacy.laser_enable, None);
    }

    #[test]
    fn optics_options_are_optional() {
        let ifaces = interfaces(&juniper::parse_braces(
            "interfaces {\n    et-0/0/0 {\n        disable;\n    }\n}\n",
        ));
        assert!(ifaces[0].optics_options.is_none());
    }
}
