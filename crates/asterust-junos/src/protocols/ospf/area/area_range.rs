//! `area-range`
//! `protocols { ospf { area <id> { area-range <prefix> [exact] [restrict] [override-metric <n>]; } } }`.
//!
//! # See also
//!
//! - [area-range](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/area-range-edit-protocols-ospf.html)

use crate::{active_children, named_children, split_name, value_after};
use asterust::config::ConfigLine;

/// An `area-range` summary, in an area or inside its `nssa`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfAreaRange {
    pub prefix: String,
    pub exact: bool,
    /// Suppresses the summary.
    pub restrict: bool,
    pub override_metric: Option<String>,
}

/// The `area-range` entries of `parent`; the options may be written in any order.
pub(super) fn parse_area_ranges(parent: &ConfigLine) -> Vec<OspfAreaRange> {
    named_children(parent, "area-range")
        .map(|(arg, range)| {
            let (prefix, rest) = split_name(arg);
            let words: Vec<&str> = rest
                .split_whitespace()
                .chain(active_children(range).flat_map(|child| child.text.split_whitespace()))
                .collect();
            OspfAreaRange {
                prefix: prefix.to_string(),
                exact: words.contains(&"exact"),
                restrict: words.contains(&"restrict"),
                override_metric: value_after(&words, "override-metric"),
            }
        })
        .collect()
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
            area-range 10.0.0.0/8 exact;
            area-range 192.168.0.0/16 restrict;
            area-range 172.16.0.0/12 override-metric 5 exact;
            area-range 203.0.113.0/24;
        }
    }
}
"#;

    #[test]
    fn reads_the_options_in_any_order() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let ranges = &ospf.areas[0].area_ranges;
        assert_eq!(ranges.len(), 4);
        assert_eq!(ranges[0].prefix, "10.0.0.0/8");
        assert!(ranges[0].exact);
        assert!(!ranges[0].restrict);
        assert!(ranges[1].restrict);
        assert!(ranges[2].exact);
        assert_eq!(ranges[2].override_metric.as_deref(), Some("5"));
        assert!(!ranges[3].exact);
        assert_eq!(ranges[3].override_metric, None);
    }
}
