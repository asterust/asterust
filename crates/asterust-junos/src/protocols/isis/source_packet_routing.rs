//! `source-packet-routing`
//! `protocols { isis { source-packet-routing { ... } } }`.
//!
//! # See also
//!
//! - [source-packet-routing (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/source-packet-routing-edit-protocols-isis.html)
//! - [srgb (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/srgb-edit-protocols-isis-source-packet-routing.html)
//! - [strict-asla-based-flex-algorithm](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/protocols-isis-source-packet-routing-strict-asla-based-flex-algorithm.html)
//! - [ultimate-hop-popping (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/ultimate-hop-popping-edit-protocols-isis-source-packet-routing.html)

use crate::{child_block, has_leaf, statement_words, value_after};
use asterust::config::ConfigLine;

/// The IS-IS `source-packet-routing` (SPRING) statement. The `flex-algorithm`, `srv6`, `udp-tunneling`,
/// `sensor-based-stats` and `traffic-statistics` children are not extracted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct IsisSourcePacketRouting {
    pub disable: bool,
    pub explicit_null: bool,
    pub no_strict_spf: bool,
    pub no_binding_sid_leaking: bool,
    pub use_flex_algorithm_metric_always: bool,
    pub strict_asla_based_flex_algorithm: bool,
    pub ultimate_hop_popping: bool,
    /// `node-segment { ipv4-index ...; }`.
    pub node_segment_ipv4_index: Option<String>,
    pub node_segment_ipv6_index: Option<String>,
    pub node_segment_index_range: Option<String>,
    /// `srgb { start-label ...; }`.
    pub srgb_start_label: Option<String>,
    pub srgb_index_range: Option<String>,
    /// Milliseconds, from `adjacency-segment { hold-time ...; }`.
    pub adjacency_segment_hold_time: Option<String>,
}

pub(super) fn parse_source_packet_routing(isis: &ConfigLine) -> Option<IsisSourcePacketRouting> {
    let spr = child_block(isis, "source-packet-routing")?;
    let node = statement_words(spr, "node-segment");
    let srgb = statement_words(spr, "srgb");
    let adjacency = statement_words(spr, "adjacency-segment");
    Some(IsisSourcePacketRouting {
        disable: has_leaf(spr, "disable"),
        explicit_null: has_leaf(spr, "explicit-null"),
        no_strict_spf: has_leaf(spr, "no-strict-spf"),
        no_binding_sid_leaking: has_leaf(spr, "no-binding-sid-leaking"),
        use_flex_algorithm_metric_always: has_leaf(spr, "use-flex-algorithm-metric-always"),
        strict_asla_based_flex_algorithm: has_leaf(spr, "strict-asla-based-flex-algorithm"),
        ultimate_hop_popping: has_leaf(spr, "ultimate-hop-popping"),
        node_segment_ipv4_index: node.as_ref().and_then(|words| value_after(words, "ipv4-index")),
        node_segment_ipv6_index: node.as_ref().and_then(|words| value_after(words, "ipv6-index")),
        node_segment_index_range: node.as_ref().and_then(|words| value_after(words, "index-range")),
        srgb_start_label: srgb.as_ref().and_then(|words| value_after(words, "start-label")),
        srgb_index_range: srgb.as_ref().and_then(|words| value_after(words, "index-range")),
        adjacency_segment_hold_time: adjacency.and_then(|words| value_after(&words, "hold-time")),
    })
}

#[cfg(test)]
mod tests {
    use crate::protocols::isis::isis;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const BLOCKS: &str = r#"
protocols {
    isis {
        source-packet-routing {
            explicit-null;
            ultimate-hop-popping;
            strict-asla-based-flex-algorithm;
            adjacency-segment {
                hold-time 200000;
            }
            node-segment {
                ipv4-index 7;
                ipv6-index 8;
                index-range 1000;
            }
            srgb {
                start-label 16000;
                index-range 8000;
            }
        }
    }
}
"#;

    const ONE_LINE: &str = r#"
protocols {
    isis {
        source-packet-routing {
            disable;
            node-segment ipv4-index 3;
            srgb start-label 800000 index-range 80000;
        }
    }
}
"#;

    #[test]
    fn reads_the_block_forms() {
        let spr = isis(&juniper::parse_braces(BLOCKS))
            .unwrap()
            .source_packet_routing
            .unwrap();
        assert!(spr.explicit_null);
        assert!(spr.ultimate_hop_popping);
        assert!(spr.strict_asla_based_flex_algorithm);
        assert_eq!(spr.adjacency_segment_hold_time.as_deref(), Some("200000"));
        assert_eq!(spr.node_segment_ipv4_index.as_deref(), Some("7"));
        assert_eq!(spr.node_segment_ipv6_index.as_deref(), Some("8"));
        assert_eq!(spr.node_segment_index_range.as_deref(), Some("1000"));
        assert_eq!(spr.srgb_start_label.as_deref(), Some("16000"));
        assert_eq!(spr.srgb_index_range.as_deref(), Some("8000"));
        assert!(!spr.disable);
    }

    #[test]
    fn reads_the_one_line_forms() {
        let spr = isis(&juniper::parse_braces(ONE_LINE))
            .unwrap()
            .source_packet_routing
            .unwrap();
        assert!(spr.disable);
        assert_eq!(spr.node_segment_ipv4_index.as_deref(), Some("3"));
        assert_eq!(spr.node_segment_ipv6_index, None);
        assert_eq!(spr.srgb_start_label.as_deref(), Some("800000"));
        assert_eq!(spr.srgb_index_range.as_deref(), Some("80000"));
    }
}
