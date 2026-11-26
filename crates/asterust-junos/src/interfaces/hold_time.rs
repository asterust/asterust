//! `hold-time`
//! `interfaces { <name> { hold-time up <ms> down <ms>; } }`.
//!
//! # See also
//!
//! - [hold-time (ae interface)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/hold-time-edit-interfaces.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The `hold-time` interface damping timers, in milliseconds.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct HoldTime {
    pub up: Option<String>,
    pub down: Option<String>,
}

/// The `hold-time` of `interface`, written on one line or as a block.
pub(super) fn parse_hold_time(interface: &ConfigLine) -> Option<HoldTime> {
    let words = statement_words(interface, "hold-time")?;
    Some(HoldTime {
        up: value_after(&words, "up"),
        down: value_after(&words, "down"),
    })
}

#[cfg(test)]
mod tests {
    use crate::interfaces::interfaces;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_hold_time_on_one_line() {
        let interfaces = interfaces(&juniper::parse_braces(
            "interfaces {\n    xe-0/0/0 {\n        hold-time up 5000 down 100;\n    }\n}\n",
        ));
        let hold_time = interfaces[0].hold_time.as_ref().unwrap();
        assert_eq!(hold_time.up.as_deref(), Some("5000"));
        assert_eq!(hold_time.down.as_deref(), Some("100"));
    }

    #[test]
    fn extracts_hold_time_block() {
        let interfaces = interfaces(&juniper::parse_braces(
            "interfaces {\n    xe-0/0/0 {\n        hold-time {\n            up 2000;\n        }\n    }\n}\n",
        ));
        let hold_time = interfaces[0].hold_time.as_ref().unwrap();
        assert_eq!(hold_time.up.as_deref(), Some("2000"));
        assert_eq!(hold_time.down, None);
    }

    #[test]
    fn hold_time_is_optional() {
        let interfaces = interfaces(&juniper::parse_braces(
            "interfaces {\n    xe-0/0/0 {\n        disable;\n    }\n}\n",
        ));
        assert!(interfaces[0].hold_time.is_none());
    }
}
