//! `stub`
//! `protocols { ospf { area <id> { stub default-metric <n> no-summaries; } } }`.
//!
//! # See also
//!
//! - [stub](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/stub-edit-protocols-ospf.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The `stub` statement of an area. A bare `stub;` leaves every field unset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfStub {
    /// The metric of the default route the ABR injects.
    pub default_metric: Option<String>,
    /// A totally stubby area.
    pub no_summaries: bool,
    pub summaries: bool,
}

/// Reads the one-line form and the block form.
pub(super) fn parse_stub(area: &ConfigLine) -> Option<OspfStub> {
    let words = statement_words(area, "stub")?;
    Some(OspfStub {
        default_metric: value_after(&words, "default-metric"),
        no_summaries: words.contains(&"no-summaries"),
        summaries: words.contains(&"summaries"),
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
        area 0.0.0.1 {
            stub default-metric 10 no-summaries;
        }
        area 0.0.0.2 {
            stub;
        }
        area 0.0.0.3 {
            stub {
                default-metric 7;
                summaries;
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_one_line_form() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let stub = ospf.areas[0].stub.as_ref().unwrap();
        assert_eq!(stub.default_metric.as_deref(), Some("10"));
        assert!(stub.no_summaries);
        assert!(!stub.summaries);
    }

    #[test]
    fn reads_the_bare_and_block_forms() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        assert_eq!(ospf.areas[1].stub.as_ref().unwrap().default_metric, None);
        let block = ospf.areas[2].stub.as_ref().unwrap();
        assert_eq!(block.default_metric.as_deref(), Some("7"));
        assert!(block.summaries);
    }
}
