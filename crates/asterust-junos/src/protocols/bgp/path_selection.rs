//! `path-selection`
//! `protocols { bgp { path-selection { ... } } }`.
//!
//! # See also
//!
//! - [path-selection](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/path-selection-edit-protocols-bgp.html)

use crate::{arg_of, child_block, statement_words};
use asterust::config::ConfigLine;

/// The BGP `path-selection` settings, valid only directly under `bgp`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct PathSelection {
    pub always_compare_med: bool,
    pub cisco_non_deterministic: bool,
    pub external_router_id: bool,
    pub as_path_ignore: bool,
    pub l2vpn_use_bgp_rules: bool,
    /// `med-plus-igp { igp-multiplier ...; }`.
    pub med_plus_igp: bool,
    pub igp_multiplier: Option<String>,
    pub med_multiplier: Option<String>,
}

pub(super) fn parse_path_selection(line: &ConfigLine) -> Option<PathSelection> {
    let words = statement_words(line, "path-selection")?;
    let med_plus_igp = child_block(child_block(line, "path-selection")?, "med-plus-igp");
    Some(PathSelection {
        always_compare_med: words.contains(&"always-compare-med"),
        cisco_non_deterministic: words.contains(&"cisco-non-deterministic"),
        external_router_id: words.contains(&"external-router-id"),
        as_path_ignore: words.contains(&"as-path-ignore"),
        l2vpn_use_bgp_rules: words.contains(&"l2vpn-use-bgp-rules"),
        med_plus_igp: words.contains(&"med-plus-igp"),
        igp_multiplier: med_plus_igp
            .and_then(|block| arg_of(block, "igp-multiplier"))
            .map(str::to_string),
        med_multiplier: med_plus_igp
            .and_then(|block| arg_of(block, "med-multiplier"))
            .map(str::to_string),
    })
}

#[cfg(test)]
mod tests {
    use crate::protocols::bgp::bgp;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    bgp {
        path-selection {
            always-compare-med;
            as-path-ignore;
            med-plus-igp {
                igp-multiplier 2;
                med-multiplier 3;
            }
        }
        multipath-build-priority {
            low;
        }
    }
}
"#;

    #[test]
    fn reads_selection_rules_and_med_plus_igp() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let selection = bgp.path_selection.unwrap();
        assert!(selection.always_compare_med);
        assert!(selection.as_path_ignore);
        assert!(!selection.cisco_non_deterministic);
        assert!(selection.med_plus_igp);
        assert_eq!(selection.igp_multiplier.as_deref(), Some("2"));
        assert_eq!(selection.med_multiplier.as_deref(), Some("3"));
        assert_eq!(bgp.multipath_build_priority.as_deref(), Some("low"));
    }
}
