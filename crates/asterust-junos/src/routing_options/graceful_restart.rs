//! `graceful-restart`
//! `routing-options { graceful-restart { ... } }`.
//!
//! # See also
//!
//! - [graceful-restart (Enabling Globally)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/graceful-restart-edit-routing-options.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The global `graceful-restart` settings. A bare `graceful-restart;` leaves every field unset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct GracefulRestart {
    pub disable: bool,
    pub helper_disable: bool,
    pub maximum_helper_recovery_time: Option<String>,
    pub maximum_helper_restart_time: Option<String>,
    pub notify_duration: Option<String>,
    pub recovery_time: Option<String>,
    pub restart_duration: Option<String>,
    pub stale_routes_time: Option<String>,
}

pub(super) fn parse_graceful_restart(root: &ConfigLine) -> Option<GracefulRestart> {
    let words = statement_words(root, "graceful-restart")?;
    Some(GracefulRestart {
        disable: words.contains(&"disable"),
        helper_disable: words.contains(&"helper-disable"),
        maximum_helper_recovery_time: value_after(&words, "maximum-helper-recovery-time"),
        maximum_helper_restart_time: value_after(&words, "maximum-helper-restart-time"),
        notify_duration: value_after(&words, "notify-duration"),
        recovery_time: value_after(&words, "recovery-time"),
        restart_duration: value_after(&words, "restart-duration"),
        stale_routes_time: value_after(&words, "stale-routes-time"),
    })
}

#[cfg(test)]
mod tests {
    use crate::routing_options::routing_options;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_graceful_restart_block() {
        let options = routing_options(&juniper::parse_braces(
            r#"
routing-options {
    graceful-restart {
        restart-duration 180;
        stale-routes-time 300;
        recovery-time 120;
        notify-duration 30;
        maximum-helper-recovery-time 200;
        maximum-helper-restart-time 400;
        helper-disable;
    }
}
"#,
        ))
        .unwrap();
        let restart = options.graceful_restart.unwrap();
        assert_eq!(restart.restart_duration.as_deref(), Some("180"));
        assert_eq!(restart.stale_routes_time.as_deref(), Some("300"));
        assert_eq!(restart.recovery_time.as_deref(), Some("120"));
        assert_eq!(restart.notify_duration.as_deref(), Some("30"));
        assert_eq!(restart.maximum_helper_recovery_time.as_deref(), Some("200"));
        assert_eq!(restart.maximum_helper_restart_time.as_deref(), Some("400"));
        assert!(restart.helper_disable);
        assert!(!restart.disable);
    }

    #[test]
    fn bare_graceful_restart_is_enabled_with_defaults() {
        let options = routing_options(&juniper::parse_braces("routing-options {\n    graceful-restart;\n}\n")).unwrap();
        let restart = options.graceful_restart.unwrap();
        assert!(!restart.disable);
        assert_eq!(restart.restart_duration, None);
    }

    #[test]
    fn graceful_restart_can_be_disabled() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    graceful-restart {\n        disable;\n    }\n}\n",
        ))
        .unwrap();
        assert!(options.graceful_restart.unwrap().disable);
    }

    #[test]
    fn no_graceful_restart_is_none() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    router-id 192.0.2.1;\n}\n",
        ))
        .unwrap();
        assert!(options.graceful_restart.is_none());
    }
}
