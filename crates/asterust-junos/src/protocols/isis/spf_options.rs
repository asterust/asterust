//! `spf-options`
//! `protocols { isis { spf-options { delay <ms>; holddown <ms>; rapid-runs <n>; } } }`.
//!
//! # See also
//!
//! - [spf-options (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/spf-options-edit-protocols-isis.html)
//! - [microloop-avoidance](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/post-convergence-path-edit-protocols-isis-spf-options-microloop-avoidance.html)
//! - [multipath (SPF Options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/multipath-edit-protocols-isis-spf-options.html)
//! - [Understanding Weighted ECMP Traffic Distribution on One-Hop IS-IS Neighbors](https://networkingdocs.hpe.com/documentation/us/en/software/junos/is-is/topics/concept/wecmp-for-one-hop-isis-neighbors-overview.html)

use crate::{child_block, statement_words, value_after};
use asterust::config::ConfigLine;

/// The SPF throttling and path options of `spf-options`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct IsisSpfOptions {
    /// Milliseconds.
    pub delay: Option<String>,
    /// Milliseconds.
    pub holddown: Option<String>,
    pub rapid_runs: Option<String>,
    /// A `microloop-avoidance` block is configured.
    pub microloop_avoidance: bool,
    /// Milliseconds, from `microloop-avoidance { post-convergence-path { delay ...; } }`.
    pub microloop_avoidance_delay: Option<String>,
    pub microloop_avoidance_maximum_labels: Option<String>,
    pub microloop_avoidance_maximum_srv6_sids: Option<String>,
    /// A `multipath` block is configured.
    pub multipath: bool,
    /// `multipath { weighted { one-hop; } }`: weighted ECMP across one-hop neighbors.
    pub multipath_weighted_one_hop: bool,
}

pub(super) fn parse_spf_options(isis: &ConfigLine) -> Option<IsisSpfOptions> {
    let words = statement_words(isis, "spf-options")?;
    let microloop = child_block(isis, "spf-options").and_then(|spf| child_block(spf, "microloop-avoidance"));
    let path = microloop.and_then(|microloop| statement_words(microloop, "post-convergence-path"));
    let multipath = child_block(isis, "spf-options").and_then(|spf| child_block(spf, "multipath"));
    let weighted = multipath.and_then(|multipath| statement_words(multipath, "weighted"));
    Some(IsisSpfOptions {
        delay: value_after(&words, "delay"),
        holddown: value_after(&words, "holddown"),
        rapid_runs: value_after(&words, "rapid-runs"),
        microloop_avoidance: microloop.is_some(),
        microloop_avoidance_delay: path.as_ref().and_then(|path| value_after(path, "delay")),
        microloop_avoidance_maximum_labels: path.as_ref().and_then(|path| value_after(path, "maximum-labels")),
        microloop_avoidance_maximum_srv6_sids: path.as_ref().and_then(|path| value_after(path, "maximum-srv6-sids")),
        multipath: multipath.is_some(),
        multipath_weighted_one_hop: weighted.is_some_and(|weighted| weighted.contains(&"one-hop")),
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
        spf-options {
            delay 50;
            holddown 5000;
            rapid-runs 3;
            microloop-avoidance {
                post-convergence-path {
                    delay 60000;
                    maximum-labels 5;
                    maximum-srv6-sids 4;
                }
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_throttle_timers_and_microloop_avoidance() {
        let spf = isis(&juniper::parse_braces(CONFIG)).unwrap().spf_options.unwrap();
        assert_eq!(spf.delay.as_deref(), Some("50"));
        assert_eq!(spf.holddown.as_deref(), Some("5000"));
        assert_eq!(spf.rapid_runs.as_deref(), Some("3"));
        assert!(spf.microloop_avoidance);
        assert_eq!(spf.microloop_avoidance_delay.as_deref(), Some("60000"));
        assert_eq!(spf.microloop_avoidance_maximum_labels.as_deref(), Some("5"));
        assert_eq!(spf.microloop_avoidance_maximum_srv6_sids.as_deref(), Some("4"));
    }

    #[test]
    fn leaves_unwritten_timers_unset() {
        let config = juniper::parse_braces(
            "protocols {\n    isis {\n        spf-options {\n            delay 100;\n        }\n    }\n}\n",
        );
        let spf = isis(&config).unwrap().spf_options.unwrap();
        assert_eq!(spf.delay.as_deref(), Some("100"));
        assert_eq!(spf.holddown, None);
        assert!(!spf.microloop_avoidance);
        assert!(!spf.multipath);
    }

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const MULTIPATH: &str = r#"
protocols {
    isis {
        spf-options {
            multipath {
                weighted {
                    one-hop;
                }
            }
        }
    }
}
"#;

    #[test]
    fn reads_weighted_one_hop_multipath() {
        let spf = isis(&juniper::parse_braces(MULTIPATH)).unwrap().spf_options.unwrap();
        assert!(spf.multipath);
        assert!(spf.multipath_weighted_one_hop);
        assert_eq!(spf.delay, None);
    }
}
