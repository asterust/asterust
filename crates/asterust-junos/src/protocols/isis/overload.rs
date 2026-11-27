//! `overload`
//! `protocols { isis { overload { ... } } }`.
//!
//! # See also
//!
//! - [overload (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/overload-edit-protocols-isis.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The IS-IS `overload` statement. A bare `overload;` leaves every field unset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct IsisOverload {
    /// Seconds.
    pub timeout: Option<String>,
    pub advertise_high_metrics: bool,
    pub allow_route_leaking: bool,
    pub external_prefixes: bool,
    pub internal_prefixes: bool,
}

pub(super) fn parse_overload(isis: &ConfigLine) -> Option<IsisOverload> {
    let words = statement_words(isis, "overload")?;
    Some(IsisOverload {
        timeout: value_after(&words, "timeout"),
        advertise_high_metrics: words.contains(&"advertise-high-metrics"),
        allow_route_leaking: words.contains(&"allow-route-leaking"),
        external_prefixes: words.contains(&"external-prefixes"),
        internal_prefixes: words.contains(&"internal-prefixes"),
    })
}

#[cfg(test)]
mod tests {
    use crate::protocols::isis::isis;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const FULL: &str = r#"
protocols {
    isis {
        overload {
            timeout 300;
            advertise-high-metrics;
            allow-route-leaking;
            external-prefixes;
        }
    }
}
"#;

    #[test]
    fn reads_the_timeout_and_flags() {
        let overload = isis(&juniper::parse_braces(FULL)).unwrap().overload.unwrap();
        assert_eq!(overload.timeout.as_deref(), Some("300"));
        assert!(overload.advertise_high_metrics);
        assert!(overload.allow_route_leaking);
        assert!(overload.external_prefixes);
        assert!(!overload.internal_prefixes);
    }

    #[test]
    fn reads_the_bare_form_and_absence() {
        let bare = juniper::parse_braces("protocols {\n    isis {\n        overload;\n    }\n}\n");
        let overload = isis(&bare).unwrap().overload.unwrap();
        assert_eq!(overload.timeout, None);
        assert!(!overload.allow_route_leaking);

        let none = juniper::parse_braces("protocols {\n    isis {\n        interface ge-0/0/0.0;\n    }\n}\n");
        assert_eq!(isis(&none).unwrap().overload, None);
    }
}
