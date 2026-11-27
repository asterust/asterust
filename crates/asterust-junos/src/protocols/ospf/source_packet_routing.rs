//! `source-packet-routing`
//! `protocols { ospf { source-packet-routing { ... } } }`.
//!
//! # See also
//!
//! - [source-packet-routing (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/source-packet-routing-edit-protocols-ospf.html)
//! - [node-segment (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/node-segment-edit-protocols-ospf-source-packet-routing.html)
//! - [ldp-stitching (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/protocols-ospf-source-packet-routing-ldp-stitching.html)
//! - [Configuring Segment Routing Microloop Avoidance in OSPFv2 Networks](https://networkingdocs.hpe.com/documentation/us/en/software/junos/segment-routing/topics/example/ospfv2-microloop-avoidance-configuring.html)

use crate::{arg_of, args_of, child_block, has_leaf, list_values, statement_words, value_after};
use asterust::config::ConfigLine;

/// The OSPF `source-packet-routing` (SPRING) statement. The `flex-algorithm` participation is `flex_algorithms`; the definitions are in `routing-options`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfSourcePacketRouting {
    pub disable: bool,
    pub explicit_null: bool,
    pub ldp_stitching: bool,
    pub sensor_based_stats: bool,
    pub strict_asla_based_flex_algorithm: bool,
    /// `node-segment { ipv4-index ...; }`.
    pub node_segment_ipv4_index: Option<String>,
    /// `node-segment { index-range ...; }`.
    pub node_segment_index_range: Option<String>,
    /// `srgb { start-label ...; }`.
    pub srgb_start_label: Option<String>,
    /// `srgb { index-range ...; }`.
    pub srgb_index_range: Option<String>,
    /// Milliseconds, from `adjacency-segment { hold-time ...; }`.
    pub adjacency_segment_hold_time: Option<String>,
    /// The argument of `prefix-segment`, as the Segment Routing User Guide example writes it.
    pub prefix_segment: Option<String>,
    /// The flexible algorithm ids this router participates in, from `flex-algorithm 128;` or `flex-algorithm [ 128 129 ];`.
    pub flex_algorithms: Vec<String>,
}

pub(super) fn parse_source_packet_routing(ospf: &ConfigLine) -> Option<OspfSourcePacketRouting> {
    let spr = child_block(ospf, "source-packet-routing")?;
    let node = statement_words(spr, "node-segment");
    let srgb = statement_words(spr, "srgb");
    let adjacency = statement_words(spr, "adjacency-segment");
    Some(OspfSourcePacketRouting {
        disable: has_leaf(spr, "disable"),
        explicit_null: has_leaf(spr, "explicit-null"),
        ldp_stitching: has_leaf(spr, "ldp-stitching"),
        sensor_based_stats: has_leaf(spr, "sensor-based-stats"),
        strict_asla_based_flex_algorithm: has_leaf(spr, "strict-asla-based-flex-algorithm"),
        node_segment_ipv4_index: node.as_ref().and_then(|words| value_after(words, "ipv4-index")),
        node_segment_index_range: node.as_ref().and_then(|words| value_after(words, "index-range")),
        srgb_start_label: srgb.as_ref().and_then(|words| value_after(words, "start-label")),
        srgb_index_range: srgb.as_ref().and_then(|words| value_after(words, "index-range")),
        adjacency_segment_hold_time: adjacency.and_then(|words| value_after(&words, "hold-time")),
        prefix_segment: arg_of(spr, "prefix-segment").map(str::to_string),
        flex_algorithms: args_of(spr, "flex-algorithm")
            .into_iter()
            .flat_map(list_values)
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use crate::protocols::ospf::ospf;
    use asterust::config::juniper;

    // Adapted from "Configuring Segment Routing Microloop Avoidance in OSPFv2 Networks" (Segment Routing User
    // Guide), Device R0 `show protocols` (OSPF part), plus a second block written from the CLI Reference; see the
    // README "Sources".
    const CONFIG: &str = r#"
protocols {
    ospf {
        source-packet-routing {
            prefix-segment prefix-sid;
            node-segment ipv4-index 0;
            srgb start-label 800000 index-range 80000;
        }
    }
}
"#;

    const BLOCKS: &str = r#"
protocols {
    ospf {
        source-packet-routing {
            explicit-null;
            ldp-stitching;
            adjacency-segment {
                hold-time 200000;
            }
            node-segment {
                ipv4-index 7;
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

    #[test]
    fn reads_the_one_line_forms_of_the_guide_example() {
        let spr = ospf(&juniper::parse_braces(CONFIG))
            .unwrap()
            .source_packet_routing
            .unwrap();
        assert_eq!(spr.prefix_segment.as_deref(), Some("prefix-sid"));
        assert_eq!(spr.node_segment_ipv4_index.as_deref(), Some("0"));
        assert_eq!(spr.srgb_start_label.as_deref(), Some("800000"));
        assert_eq!(spr.srgb_index_range.as_deref(), Some("80000"));
        assert!(!spr.disable);
    }

    #[test]
    fn reads_the_block_forms() {
        let spr = ospf(&juniper::parse_braces(BLOCKS))
            .unwrap()
            .source_packet_routing
            .unwrap();
        assert!(spr.explicit_null);
        assert!(spr.ldp_stitching);
        assert_eq!(spr.adjacency_segment_hold_time.as_deref(), Some("200000"));
        assert_eq!(spr.node_segment_ipv4_index.as_deref(), Some("7"));
        assert_eq!(spr.node_segment_index_range.as_deref(), Some("1000"));
        assert_eq!(spr.srgb_start_label.as_deref(), Some("16000"));
        assert_eq!(spr.srgb_index_range.as_deref(), Some("8000"));
        assert_eq!(spr.prefix_segment, None);
        assert!(spr.flex_algorithms.is_empty());
    }

    // Adapted from "How to Configure Flexible Algorithms in OSPF for Segment Routing Traffic Engineering" (OSPF User
    // Guide), Device R0 `show protocols` (OSPF part), plus entries written from the CLI Reference; see the README
    // "Sources".
    const FLEX: &str = r#"
protocols {
    ospf {
        source-packet-routing {
            prefix-segment prefix-sid;
            srgb start-label 80000 index-range 5000;
            flex-algorithm [ 128 129 ];
            flex-algorithm 130;
            flex-algorithm 131 {
                color 1310;
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_flex_algorithm_participation() {
        let spr = ospf(&juniper::parse_braces(FLEX))
            .unwrap()
            .source_packet_routing
            .unwrap();
        assert_eq!(spr.flex_algorithms, ["128", "129", "130", "131"]);
        assert_eq!(spr.srgb_start_label.as_deref(), Some("80000"));
    }
}
