//! `spf-options`
//! `protocols { ospf { spf-options { delay <ms>; holddown <ms>; rapid-runs <n>; } } }`.
//!
//! # See also
//!
//! - [spf-options (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/spf-options-edit-protocols-ospf.html)
//! - [microloop-avoidance (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/post-convergence-path-edit-protocols-ospf-spf-options-microloop-avoidance.html)
//! - [Configuring Segment Routing Microloop Avoidance in OSPFv2 Networks](https://networkingdocs.hpe.com/documentation/us/en/software/junos/segment-routing/topics/example/ospfv2-microloop-avoidance-configuring.html)

use crate::{child_block, statement_words, value_after};
use asterust::config::ConfigLine;

/// The SPF throttling of `spf-options`. The `multipath` child is not extracted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfSpfOptions {
    /// Milliseconds.
    pub delay: Option<String>,
    /// Milliseconds.
    pub holddown: Option<String>,
    pub rapid_runs: Option<String>,
    /// A `microloop-avoidance` block is configured (22.1 and later).
    pub microloop_avoidance: bool,
    /// Milliseconds, from `microloop-avoidance { post-convergence-path { delay ...; } }`.
    pub microloop_avoidance_delay: Option<String>,
    /// From `microloop-avoidance { post-convergence-path { maximum-labels ...; } }`.
    pub microloop_avoidance_maximum_labels: Option<String>,
}

pub(super) fn parse_spf_options(ospf: &ConfigLine) -> Option<OspfSpfOptions> {
    let words = statement_words(ospf, "spf-options")?;
    let microloop = child_block(ospf, "spf-options").and_then(|spf| child_block(spf, "microloop-avoidance"));
    let path = microloop.and_then(|microloop| statement_words(microloop, "post-convergence-path"));
    Some(OspfSpfOptions {
        delay: value_after(&words, "delay"),
        holddown: value_after(&words, "holddown"),
        rapid_runs: value_after(&words, "rapid-runs"),
        microloop_avoidance: microloop.is_some(),
        microloop_avoidance_delay: path.as_ref().and_then(|path| value_after(path, "delay")),
        microloop_avoidance_maximum_labels: path.as_ref().and_then(|path| value_after(path, "maximum-labels")),
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
        spf-options {
            delay 50;
            holddown 5000;
            rapid-runs 3;
        }
    }
}
"#;

    #[test]
    fn reads_the_throttle_timers() {
        let spf = ospf(&juniper::parse_braces(CONFIG)).unwrap().spf_options.unwrap();
        assert_eq!(spf.delay.as_deref(), Some("50"));
        assert_eq!(spf.holddown.as_deref(), Some("5000"));
        assert_eq!(spf.rapid_runs.as_deref(), Some("3"));
    }

    #[test]
    fn leaves_unwritten_timers_unset() {
        let config = juniper::parse_braces(
            "protocols {\n    ospf {\n        spf-options {\n            delay 100;\n        }\n    }\n}\n",
        );
        let spf = ospf(&config).unwrap().spf_options.unwrap();
        assert_eq!(spf.delay.as_deref(), Some("100"));
        assert_eq!(spf.holddown, None);
        assert!(!spf.microloop_avoidance);
    }

    // Adapted from "Configuring Segment Routing Microloop Avoidance in OSPFv2 Networks" (Segment Routing User
    // Guide), Device R0 `show protocols` (OSPF part), with `maximum-labels` added from the CLI Reference; see the
    // README "Sources".
    const MICROLOOP: &str = r#"
protocols {
    ospf {
        spf-options {
            microloop-avoidance {
                post-convergence-path {
                    delay 60000;
                    maximum-labels 5;
                }
            }
        }
    }
}
"#;

    #[test]
    fn reads_microloop_avoidance() {
        let spf = ospf(&juniper::parse_braces(MICROLOOP)).unwrap().spf_options.unwrap();
        assert!(spf.microloop_avoidance);
        assert_eq!(spf.microloop_avoidance_delay.as_deref(), Some("60000"));
        assert_eq!(spf.microloop_avoidance_maximum_labels.as_deref(), Some("5"));
        assert_eq!(spf.delay, None);
    }
}
