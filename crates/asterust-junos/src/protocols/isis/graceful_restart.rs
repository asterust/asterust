//! `graceful-restart`
//! `protocols { isis { graceful-restart { ... } } }`.
//!
//! # See also
//!
//! - [graceful-restart (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/graceful-restart-edit-protocols-isis.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The IS-IS `graceful-restart` statement. A bare `graceful-restart;` leaves every field unset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct IsisGracefulRestart {
    pub disable: bool,
    pub helper_disable: bool,
    /// Seconds.
    pub restart_duration: Option<String>,
}

pub(super) fn parse_graceful_restart(isis: &ConfigLine) -> Option<IsisGracefulRestart> {
    let words = statement_words(isis, "graceful-restart")?;
    Some(IsisGracefulRestart {
        disable: words.contains(&"disable"),
        helper_disable: words.contains(&"helper-disable"),
        restart_duration: value_after(&words, "restart-duration"),
    })
}

#[cfg(test)]
mod tests {
    use crate::protocols::isis::isis;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    isis {
        graceful-restart {
            helper-disable;
            restart-duration 60;
        }
    }
}
"#;

    #[test]
    fn reads_the_helper_and_duration() {
        let restart = isis(&juniper::parse_braces(CONFIG)).unwrap().graceful_restart.unwrap();
        assert!(restart.helper_disable);
        assert_eq!(restart.restart_duration.as_deref(), Some("60"));
        assert!(!restart.disable);
    }

    #[test]
    fn reads_disable() {
        let config = juniper::parse_braces(
            "protocols {\n    isis {\n        graceful-restart {\n            disable;\n        }\n    }\n}\n",
        );
        let restart = isis(&config).unwrap().graceful_restart.unwrap();
        assert!(restart.disable);
        assert!(!restart.helper_disable);
    }
}
