//! `multipath`
//! `protocols { bgp { group <name> { multipath { ... } } } }`.
//!
//! # See also
//!
//! - [multipath (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/multipath-edit-protocols-bgp.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The BGP `multipath` statement. Written `multipath;` or with options in a block; the `prioritization` block is
/// not extracted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpMultipath {
    /// `multipath { disable; }`, to switch off an inherited `multipath`.
    pub disable: bool,
    /// `multiple-as`: load-balance across paths from different neighboring ASes.
    pub multiple_as: bool,
    pub allow_external_internal: bool,
    pub allow_protection: bool,
    pub list_nexthop: bool,
    /// Seconds.
    pub starving_age: Option<String>,
}

pub(super) fn parse_multipath(line: &ConfigLine) -> Option<BgpMultipath> {
    let words = statement_words(line, "multipath")?;
    Some(BgpMultipath {
        disable: words.contains(&"disable"),
        multiple_as: words.contains(&"multiple-as"),
        allow_external_internal: words.contains(&"allow-external-internal"),
        allow_protection: words.contains(&"allow-protection"),
        list_nexthop: words.contains(&"list-nexthop"),
        starving_age: value_after(&words, "starving-age"),
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
        multipath;
        group EBGP {
            multipath {
                multiple-as;
                starving-age 30;
            }
            neighbor 192.0.2.2 {
                multipath {
                    disable;
                }
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_bare_and_block_forms_at_every_level() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        assert_eq!(bgp.options.multipath.as_ref().map(|m| m.multiple_as), Some(false));

        let group = bgp.groups[0].options.multipath.as_ref().unwrap();
        assert!(group.multiple_as);
        assert_eq!(group.starving_age.as_deref(), Some("30"));
        assert!(!group.disable);

        let neighbor = bgp.groups[0].neighbors[0].options.multipath.as_ref().unwrap();
        assert!(neighbor.disable);
    }

    #[test]
    fn is_none_when_absent() {
        let config = juniper::parse_braces(
            "protocols {\n    bgp {\n        group G {\n            type external;\n        }\n    }\n}\n",
        );
        assert_eq!(bgp(&config).unwrap().groups[0].options.multipath, None);
    }
}
