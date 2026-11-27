//! `spf-options`
//! `protocols { ospf { spf-options { delay <ms>; holddown <ms>; rapid-runs <n>; } } }`.
//!
//! # See also
//!
//! - [spf-options (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/spf-options-edit-protocols-ospf.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The SPF throttling of `spf-options`. The `multipath` and `microloop-avoidance` children are not extracted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfSpfOptions {
    /// Milliseconds.
    pub delay: Option<String>,
    /// Milliseconds.
    pub holddown: Option<String>,
    pub rapid_runs: Option<String>,
}

pub(super) fn parse_spf_options(ospf: &ConfigLine) -> Option<OspfSpfOptions> {
    let words = statement_words(ospf, "spf-options")?;
    Some(OspfSpfOptions {
        delay: value_after(&words, "delay"),
        holddown: value_after(&words, "holddown"),
        rapid_runs: value_after(&words, "rapid-runs"),
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
    }
}
