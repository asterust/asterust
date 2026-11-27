//! `prefix-limit` and `accepted-prefix-limit`
//! `protocols { bgp { group <name> { family <afi> { <safi> { prefix-limit { ... } } } } } }`.
//!
//! # See also
//!
//! - [prefix-limit](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/prefix-limit-edit-protocols-bgp.html)
//! - [accepted-prefix-limit](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/accepted-prefix-limit-edit-protocols-bgp.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The limit on the prefixes received from a peer, from `prefix-limit` or `accepted-prefix-limit`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct PrefixLimit {
    pub maximum: Option<String>,
    /// `teardown` is configured, with or without a percentage.
    pub teardown: bool,
    pub teardown_percentage: Option<String>,
    /// Minutes, or `forever`.
    pub idle_timeout: Option<String>,
    pub drop_excess: bool,
    pub drop_excess_percentage: Option<String>,
    pub hide_excess: bool,
    pub hide_excess_percentage: Option<String>,
}

/// Reads `keyword { maximum <n>; teardown <percentage> idle-timeout <minutes>; }`, and the older `keyword <n>;`.
pub(super) fn parse_prefix_limit(node: &ConfigLine, keyword: &str) -> Option<PrefixLimit> {
    let words = statement_words(node, keyword)?;
    let is_number = |word: &str| word.chars().all(|c| c.is_ascii_digit());
    let percentage = |name: &str| value_after(&words, name).filter(|value| is_number(value));
    let bare_maximum = words
        .first()
        .filter(|word| is_number(word))
        .map(|word| word.to_string());
    Some(PrefixLimit {
        maximum: value_after(&words, "maximum").or(bare_maximum),
        teardown: words.contains(&"teardown"),
        teardown_percentage: percentage("teardown"),
        idle_timeout: value_after(&words, "idle-timeout"),
        drop_excess: words.contains(&"drop-excess"),
        drop_excess_percentage: percentage("drop-excess"),
        hide_excess: words.contains(&"hide-excess"),
        hide_excess_percentage: percentage("hide-excess"),
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
        group EBGP {
            family inet {
                unicast {
                    prefix-limit {
                        maximum 1000;
                        teardown 80 idle-timeout 10;
                    }
                    accepted-prefix-limit {
                        maximum 500;
                        hide-excess 90;
                    }
                }
                multicast {
                    prefix-limit {
                        maximum 10;
                        teardown idle-timeout forever;
                    }
                    accepted-prefix-limit {
                        maximum 20;
                        drop-excess;
                    }
                }
            }
            family iso-vpn {
                unicast {
                    prefix-limit 250;
                }
            }
        }
    }
}
"#;

    #[test]
    fn reads_teardown_and_idle_timeout() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let limit = bgp.groups[0].options.address_families[0].prefix_limit.as_ref().unwrap();
        assert_eq!(limit.maximum.as_deref(), Some("1000"));
        assert!(limit.teardown);
        assert_eq!(limit.teardown_percentage.as_deref(), Some("80"));
        assert_eq!(limit.idle_timeout.as_deref(), Some("10"));
        assert!(!limit.drop_excess);
    }

    #[test]
    fn reads_the_excess_actions_of_the_accepted_limit() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let families = &bgp.groups[0].options.address_families;

        let hide = families[0].accepted_prefix_limit.as_ref().unwrap();
        assert!(hide.hide_excess);
        assert_eq!(hide.hide_excess_percentage.as_deref(), Some("90"));
        assert!(!hide.teardown);

        let drop = families[1].accepted_prefix_limit.as_ref().unwrap();
        assert!(drop.drop_excess);
        assert_eq!(drop.drop_excess_percentage, None);
    }

    #[test]
    fn reads_a_bare_teardown_with_forever() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let limit = bgp.groups[0].options.address_families[1].prefix_limit.as_ref().unwrap();
        assert!(limit.teardown);
        assert_eq!(limit.teardown_percentage, None);
        assert_eq!(limit.idle_timeout.as_deref(), Some("forever"));
    }

    #[test]
    fn reads_the_older_single_number_form() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let limit = bgp.groups[0].options.address_families[2].prefix_limit.as_ref().unwrap();
        assert_eq!(limit.maximum.as_deref(), Some("250"));
    }
}
