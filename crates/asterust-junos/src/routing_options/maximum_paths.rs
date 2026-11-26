//! `maximum-paths`
//! `routing-options { maximum-paths <limit> [log-interval <s>] [log-only | threshold <n>]; }`.
//!
//! # See also
//!
//! - [maximum-paths](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/maximum-paths-edit-routing-options.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The limit on the number of routes in the routing table.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct MaximumPaths {
    pub path_limit: Option<String>,
    /// Seconds between log messages.
    pub log_interval: Option<String>,
    /// Log a message instead of enforcing the limit.
    pub log_only: bool,
    /// Percentage of the limit that triggers a warning.
    pub threshold: Option<String>,
}

pub(super) fn parse_maximum_paths(root: &ConfigLine) -> Option<MaximumPaths> {
    let words = statement_words(root, "maximum-paths")?;
    Some(MaximumPaths {
        path_limit: words.first().map(|word| word.to_string()),
        log_interval: value_after(&words, "log-interval"),
        log_only: words.contains(&"log-only"),
        threshold: value_after(&words, "threshold"),
    })
}

#[cfg(test)]
mod tests {
    use crate::routing_options::routing_options;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_limit_threshold_and_interval() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    maximum-paths 1000 threshold 80 log-interval 60;\n}\n",
        ))
        .unwrap();
        let paths = options.maximum_paths.unwrap();
        assert_eq!(paths.path_limit.as_deref(), Some("1000"));
        assert_eq!(paths.threshold.as_deref(), Some("80"));
        assert_eq!(paths.log_interval.as_deref(), Some("60"));
        assert!(!paths.log_only);
    }

    #[test]
    fn extracts_log_only() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    maximum-paths 500 log-only;\n}\n",
        ))
        .unwrap();
        let paths = options.maximum_paths.unwrap();
        assert_eq!(paths.path_limit.as_deref(), Some("500"));
        assert!(paths.log_only);
        assert_eq!(paths.threshold, None);
    }

    #[test]
    fn no_maximum_paths_is_none() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    router-id 192.0.2.1;\n}\n",
        ))
        .unwrap();
        assert!(options.maximum_paths.is_none());
    }
}
