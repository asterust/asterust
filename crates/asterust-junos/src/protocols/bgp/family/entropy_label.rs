//! `entropy-label`
//! `protocols { bgp { group <name> { family inet { labeled-unicast { entropy-label { ... } } } } } }`.
//!
//! # See also
//!
//! - [entropy-label (protocols bgp)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/entropy-label-edit-protocols-bgp-family-inet-labeled-unicast.html)
//! - [Load Balancing for a BGP Session](https://networkingdocs.hpe.com/documentation/us/en/software/junos/bgp/topics/topic-map/load-balancing-bgp-session.html)

use crate::{args_of, child_block, has_leaf, list_values};
use asterust::config::ConfigLine;

/// The `entropy-label` statement of `family inet labeled-unicast`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct EntropyLabel {
    pub elc_v2_compatible: bool,
    pub no_next_hop_validation: bool,
    /// Policies selecting the LSPs that use an entropy label.
    pub import: Vec<String>,
}

pub(super) fn parse_entropy_label(node: &ConfigLine) -> Option<EntropyLabel> {
    let label = child_block(node, "entropy-label")?;
    Some(EntropyLabel {
        elc_v2_compatible: has_leaf(label, "elc-v2-compatible"),
        no_next_hop_validation: has_leaf(label, "no-next-hop-validation"),
        import: args_of(label, "import").into_iter().flat_map(list_values).collect(),
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
        group LU {
            family inet {
                labeled-unicast {
                    entropy-label {
                        elc-v2-compatible;
                        import [ ENTROPY-A ENTROPY-B ];
                    }
                }
                unicast {
                    extended-nexthop;
                }
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_entropy_label_options() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let label = bgp.groups[0].options.address_families[0]
            .entropy_label
            .as_ref()
            .unwrap();
        assert!(label.elc_v2_compatible);
        assert!(!label.no_next_hop_validation);
        assert_eq!(label.import, ["ENTROPY-A", "ENTROPY-B"]);
    }

    #[test]
    fn reads_extended_nexthop() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let families = &bgp.groups[0].options.address_families;
        assert!(families[1].extended_nexthop);
        assert!(!families[0].extended_nexthop);
        assert_eq!(families[1].entropy_label, None);
    }
}
