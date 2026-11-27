//! `graceful-restart`
//! `protocols { ospf { graceful-restart { ... } } }`.
//!
//! # See also
//!
//! - [graceful-restart (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/graceful-restart-edit-protocols-ospf.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The OSPF `graceful-restart` statement. A bare `graceful-restart;` leaves every field unset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfGracefulRestart {
    pub disable: bool,
    pub helper_disable: bool,
    /// `standard`, `restart-signaling` or `both`, when written after `helper-disable`.
    pub helper_disable_mode: Option<String>,
    pub no_strict_lsa_checking: bool,
    /// Seconds.
    pub notify_duration: Option<String>,
    /// Seconds.
    pub restart_duration: Option<String>,
}

pub(super) fn parse_graceful_restart(ospf: &ConfigLine) -> Option<OspfGracefulRestart> {
    let words = statement_words(ospf, "graceful-restart")?;
    Some(OspfGracefulRestart {
        disable: words.contains(&"disable"),
        helper_disable: words.contains(&"helper-disable"),
        helper_disable_mode: value_after(&words, "helper-disable")
            .filter(|mode| matches!(mode.as_str(), "standard" | "restart-signaling" | "both")),
        no_strict_lsa_checking: words.contains(&"no-strict-lsa-checking"),
        notify_duration: value_after(&words, "notify-duration"),
        restart_duration: value_after(&words, "restart-duration"),
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
        graceful-restart {
            helper-disable both;
            no-strict-lsa-checking;
            notify-duration 20;
            restart-duration 60;
        }
    }
}
"#;

    #[test]
    fn reads_the_helper_mode_and_durations() {
        let restart = ospf(&juniper::parse_braces(CONFIG)).unwrap().graceful_restart.unwrap();
        assert!(restart.helper_disable);
        assert_eq!(restart.helper_disable_mode.as_deref(), Some("both"));
        assert!(restart.no_strict_lsa_checking);
        assert_eq!(restart.notify_duration.as_deref(), Some("20"));
        assert_eq!(restart.restart_duration.as_deref(), Some("60"));
        assert!(!restart.disable);
    }

    #[test]
    fn does_not_mistake_the_next_statement_for_a_helper_mode() {
        let config = juniper::parse_braces("protocols {\n    ospf {\n        graceful-restart {\n            helper-disable;\n            restart-duration 60;\n        }\n    }\n}\n");
        let restart = ospf(&config).unwrap().graceful_restart.unwrap();
        assert!(restart.helper_disable);
        assert_eq!(restart.helper_disable_mode, None);

        let disabled = juniper::parse_braces(
            "protocols {\n    ospf {\n        graceful-restart {\n            disable;\n        }\n    }\n}\n",
        );
        assert!(ospf(&disabled).unwrap().graceful_restart.unwrap().disable);
    }
}
