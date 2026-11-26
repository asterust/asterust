//! `autonomous-system`
//! `routing-options { autonomous-system <as> [asdot-notation] [loops <n>] [independent-domain]; }`.
//!
//! # See also
//!
//! - [autonomous-system](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/autonomous-system-edit-routing-options.html)
//! - [independent-domain](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/independent-domain-edit-routing-options.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The options of the `autonomous-system` statement. The AS number itself is
/// [`RoutingOptions::autonomous_system`](super::RoutingOptions::autonomous_system).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AutonomousSystemOptions {
    /// How many times the local AS may appear in a received AS path.
    pub loops: Option<String>,
    /// Displays 4-byte AS numbers as `high.low`.
    pub asdot_notation: bool,
    pub independent_domain: bool,
    /// The `no-attrset` option of `independent-domain`.
    pub no_attrset: bool,
}

/// Reads the options written inline (`autonomous-system 65000 loops 2;`) or as child statements.
pub(super) fn parse_autonomous_system_options(root: &ConfigLine) -> Option<AutonomousSystemOptions> {
    let words = statement_words(root, "autonomous-system")?;
    Some(AutonomousSystemOptions {
        loops: value_after(&words, "loops"),
        asdot_notation: words.contains(&"asdot-notation"),
        independent_domain: words.contains(&"independent-domain"),
        no_attrset: words.contains(&"no-attrset"),
    })
}

#[cfg(test)]
mod tests {
    use crate::routing_options::routing_options;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_inline_options() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    autonomous-system 65000 asdot-notation loops 2 independent-domain no-attrset;\n}\n",
        ))
        .unwrap();
        assert_eq!(options.autonomous_system.as_deref(), Some("65000"));
        let settings = options.autonomous_system_options.unwrap();
        assert_eq!(settings.loops.as_deref(), Some("2"));
        assert!(settings.asdot_notation);
        assert!(settings.independent_domain);
        assert!(settings.no_attrset);
    }

    #[test]
    fn extracts_block_form() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    autonomous-system 65000 {\n        loops 3;\n        independent-domain;\n    }\n}\n",
        ))
        .unwrap();
        let settings = options.autonomous_system_options.unwrap();
        assert_eq!(settings.loops.as_deref(), Some("3"));
        assert!(settings.independent_domain);
        assert!(!settings.no_attrset);
    }

    #[test]
    fn plain_autonomous_system_has_no_options() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    autonomous-system 17;\n}\n",
        ))
        .unwrap();
        let settings = options.autonomous_system_options.unwrap();
        assert_eq!(settings.loops, None);
        assert!(!settings.asdot_notation);
        assert!(!settings.independent_domain);
    }

    #[test]
    fn asdot_number_is_kept_as_written() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    autonomous-system 1.10;\n}\n",
        ))
        .unwrap();
        assert_eq!(options.autonomous_system.as_deref(), Some("1.10"));
    }

    #[test]
    fn no_autonomous_system_has_no_options() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    router-id 192.0.2.1;\n}\n",
        ))
        .unwrap();
        assert!(options.autonomous_system_options.is_none());
    }
}
