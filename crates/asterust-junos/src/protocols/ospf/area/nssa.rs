//! `nssa`
//! `protocols { ospf { area <id> { nssa { ... } } } }`.
//!
//! # See also
//!
//! - [nssa](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/nssa-edit-protocols-ospf.html)
//! - [default-lsa](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/default-lsa-edit-protocols-ospf.html)

use super::area_range::{parse_area_ranges, OspfAreaRange};
use crate::{child_block, statement_words, value_after};
use asterust::config::ConfigLine;

/// The `nssa` statement of an area. A bare `nssa;` leaves every field empty.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfNssa {
    pub no_summaries: bool,
    pub summaries: bool,
    /// From `default-lsa { default-metric ...; }`.
    pub default_metric: Option<String>,
    /// `1` or `2`, from `default-lsa { metric-type ...; }`.
    pub default_metric_type: Option<String>,
    /// `default-lsa { type-7; }`.
    pub default_type_7: bool,
    /// The `area-range` entries inside the `nssa`, which summarize Type 7 routes.
    pub area_ranges: Vec<OspfAreaRange>,
}

pub(super) fn parse_nssa(area: &ConfigLine) -> Option<OspfNssa> {
    let words = statement_words(area, "nssa")?;
    let block = child_block(area, "nssa");
    let default_lsa = block.and_then(|nssa| statement_words(nssa, "default-lsa"));
    Some(OspfNssa {
        no_summaries: words.contains(&"no-summaries"),
        summaries: words.contains(&"summaries"),
        default_metric: default_lsa.as_ref().and_then(|lsa| value_after(lsa, "default-metric")),
        default_metric_type: default_lsa.as_ref().and_then(|lsa| value_after(lsa, "metric-type")),
        default_type_7: default_lsa.is_some_and(|lsa| lsa.contains(&"type-7")),
        area_ranges: block.map(parse_area_ranges).unwrap_or_default(),
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
        area 0.0.0.9 {
            nssa {
                default-lsa {
                    default-metric 10;
                    metric-type 1;
                    type-7;
                }
                no-summaries;
                area-range 172.16.0.0/16 restrict;
            }
        }
        area 0.0.0.10 {
            nssa;
        }
        area 0.0.0.11 {
            nssa summaries;
        }
    }
}
"#;

    #[test]
    fn reads_the_default_lsa_and_ranges() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let nssa = ospf.areas[0].nssa.as_ref().unwrap();
        assert_eq!(nssa.default_metric.as_deref(), Some("10"));
        assert_eq!(nssa.default_metric_type.as_deref(), Some("1"));
        assert!(nssa.default_type_7);
        assert!(nssa.no_summaries);
        assert!(!nssa.summaries);
        assert_eq!(nssa.area_ranges.len(), 1);
        assert_eq!(nssa.area_ranges[0].prefix, "172.16.0.0/16");
        assert!(nssa.area_ranges[0].restrict);
        assert!(ospf.areas[0].area_ranges.is_empty());
    }

    #[test]
    fn reads_the_bare_and_inline_forms() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let bare = ospf.areas[1].nssa.as_ref().unwrap();
        assert_eq!(bare.default_metric, None);
        assert!(!bare.default_type_7);
        assert!(ospf.areas[2].nssa.as_ref().unwrap().summaries);
    }
}
