//! `link-degrade-monitor`
//! `interfaces { <name> { link-degrade-monitor { ... } } }`.
//!
//! # See also
//!
//! - [thresholds](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/interfaces-link-degrade-monitor-thresholds.html)
//! - [recovery](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/interfaces-link-degrade-monitor-recovery.html)

use crate::{arg_of, child_block, has_leaf};
use asterust::config::ConfigLine;

/// The BER `thresholds` of a link degrade monitor, written `1E-<n>`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct LinkDegradeThresholds {
    pub set: Option<String>,
    pub clear: Option<String>,
    pub warning_set: Option<String>,
    pub warning_clear: Option<String>,
    /// Consecutive degrade events before the action is taken.
    pub interval: Option<String>,
}

/// The `link-degrade-monitor` settings of an interface.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LinkDegradeMonitor {
    pub thresholds: Option<LinkDegradeThresholds>,
    /// `auto` or `manual`, from `recovery`.
    pub recovery_mode: Option<String>,
    /// Seconds, from `recovery { timer ...; }`.
    pub recovery_timer: Option<String>,
}

pub(super) fn parse_link_degrade_monitor(monitor: &ConfigLine) -> LinkDegradeMonitor {
    let recovery = child_block(monitor, "recovery");
    LinkDegradeMonitor {
        thresholds: child_block(monitor, "thresholds").map(|thresholds| {
            let field = |keyword: &str| arg_of(thresholds, keyword).map(str::to_string);
            LinkDegradeThresholds {
                set: field("set"),
                clear: field("clear"),
                warning_set: field("warning-set"),
                warning_clear: field("warning-clear"),
                interval: field("interval"),
            }
        }),
        recovery_mode: recovery.and_then(|recovery| {
            ["auto", "manual"]
                .iter()
                .find(|mode| has_leaf(recovery, mode))
                .map(|mode| mode.to_string())
        }),
        recovery_timer: recovery
            .and_then(|recovery| arg_of(recovery, "timer"))
            .map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use crate::interfaces::interfaces;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_link_degrade_monitor() {
        let ifaces = interfaces(&juniper::parse_braces(
            r#"
interfaces {
    et-0/0/0 {
        link-degrade-monitor {
            thresholds {
                set 1E-7;
                clear 1E-12;
                warning-set 1E-9;
                warning-clear 1E-11;
                interval 10;
            }
            recovery {
                auto;
                timer 1800;
            }
        }
    }
    et-0/0/1 {
        link-degrade-monitor {
            recovery {
                manual;
            }
        }
    }
}
"#,
        ));
        let monitor = ifaces[0].link_degrade_monitor.as_ref().unwrap();
        let thresholds = monitor.thresholds.as_ref().unwrap();
        assert_eq!(thresholds.set.as_deref(), Some("1E-7"));
        assert_eq!(thresholds.clear.as_deref(), Some("1E-12"));
        assert_eq!(thresholds.warning_set.as_deref(), Some("1E-9"));
        assert_eq!(thresholds.warning_clear.as_deref(), Some("1E-11"));
        assert_eq!(thresholds.interval.as_deref(), Some("10"));
        assert_eq!(monitor.recovery_mode.as_deref(), Some("auto"));
        assert_eq!(monitor.recovery_timer.as_deref(), Some("1800"));

        let manual = ifaces[1].link_degrade_monitor.as_ref().unwrap();
        assert!(manual.thresholds.is_none());
        assert_eq!(manual.recovery_mode.as_deref(), Some("manual"));
        assert_eq!(manual.recovery_timer, None);
    }

    #[test]
    fn link_degrade_monitor_is_optional() {
        let ifaces = interfaces(&juniper::parse_braces(
            "interfaces {\n    et-0/0/0 {\n        disable;\n    }\n}\n",
        ));
        assert!(ifaces[0].link_degrade_monitor.is_none());
    }
}
