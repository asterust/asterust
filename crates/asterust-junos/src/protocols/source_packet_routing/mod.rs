//! `source-packet-routing`
//! `protocols { source-packet-routing { segment-list <name> { ... } source-routing-path <name> { ... } } }`.
//!
//! This is segment routing traffic engineering (SR-TE), including static SRv6-TE policies. The IGP statements of the
//! same name are in [`crate::protocols::isis`] and [`crate::protocols::ospf`].
//!
//! # See also
//!
//! - [source-packet-routing (Segment Routing Traffic Engineering)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/autogen-protocols-source-packet-routing.html)
//! - [preserve-nexthop-hierarchy (SRv6 TE)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/preserve-nexthop-hierarchy-srv6-te.html)
//! - [BGP Egress Traffic Engineering](https://networkingdocs.hpe.com/documentation/us/en/software/junos/bgp/topics/topic-map/bgp-egress-traffic-engineering.html),
//!   which has "Overview of SR-TE Policy for SRv6 Tunnel" and "Example: Configuring Static SR-TE Policy for an SRv6
//!   Tunnel"

mod segment_list;
mod source_routing_path;

pub use segment_list::{SegmentHop, SegmentList};
pub use source_routing_path::{PathSegmentList, SourceRoutingPath};

use crate::{arg_of, child_block, has_leaf, named_children, top_level};
use asterust::config::Config;
use segment_list::parse_segment_list;
use source_routing_path::parse_source_routing_path;

/// The `protocols source-packet-routing` configuration. The `telemetry`, `traceoptions`, `traffic-engineering`,
/// `source-routing-path-template` and `compute-profile` statements are not extracted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct SourcePacketRouting {
    pub inherit_label_nexthops: bool,
    /// The name of the external path computing entity.
    pub lsp_external_controller: Option<String>,
    /// 1 through 16.
    pub maximum_segment_list_depth: Option<String>,
    /// 1 through 6, or 1 through 5 with `no-reduced-srh`.
    pub maximum_srv6_te_segment_list_depth: Option<String>,
    pub preference: Option<String>,
    pub preserve_nexthop_hierarchy: bool,
    /// Seconds.
    pub retry_timer: Option<String>,
    pub sbfd_frr: bool,
    pub sr_preference: Option<String>,
    pub sr_preference_override: Option<String>,
    /// SRv6 data plane for statically configured SR-TE policies.
    pub srv6: bool,
    pub tunnel_tracking: bool,
    pub segment_lists: Vec<SegmentList>,
    pub source_routing_paths: Vec<SourceRoutingPath>,
}

/// Returns the top-level `protocols source-packet-routing` block, or `None` if it is absent.
///
/// # Examples
///
/// ```
/// use asterust::config::juniper;
/// use asterust_junos::protocols::source_packet_routing::source_packet_routing;
///
/// let config = juniper::parse_braces("protocols {\n    source-packet-routing {\n        srv6;\n        segment-list SL {\n            srv6;\n            hop1 srv6-sid 2001:db8::1;\n        }\n    }\n}\n");
/// let spr = source_packet_routing(&config).unwrap();
/// assert!(spr.srv6);
/// assert_eq!(spr.segment_lists[0].hops[0].srv6_sid.as_deref(), Some("2001:db8::1"));
/// ```
pub fn source_packet_routing(config: &Config) -> Option<SourcePacketRouting> {
    let spr = child_block(top_level(config, "protocols")?, "source-packet-routing")?;
    let field = |keyword: &str| arg_of(spr, keyword).map(str::to_string);
    Some(SourcePacketRouting {
        inherit_label_nexthops: has_leaf(spr, "inherit-label-nexthops"),
        lsp_external_controller: field("lsp-external-controller"),
        maximum_segment_list_depth: field("maximum-segment-list-depth"),
        maximum_srv6_te_segment_list_depth: field("maximum-srv6-te-segment-list-depth"),
        preference: field("preference"),
        preserve_nexthop_hierarchy: has_leaf(spr, "preserve-nexthop-hierarchy"),
        retry_timer: field("retry-timer"),
        sbfd_frr: has_leaf(spr, "sbfd-frr"),
        sr_preference: field("sr-preference"),
        sr_preference_override: field("sr-preference-override"),
        srv6: has_leaf(spr, "srv6"),
        tunnel_tracking: has_leaf(spr, "tunnel-tracking"),
        segment_lists: named_children(spr, "segment-list")
            .map(|(name, list)| parse_segment_list(name, list))
            .collect(),
        source_routing_paths: named_children(spr, "source-routing-path")
            .map(|(name, path)| parse_source_routing_path(name, path))
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::juniper;

    // Device R1 `show protocols` (source-packet-routing part) of "Example: Configuring Static SR-TE Policy for an SRv6
    // Tunnel" (BGP User Guide, BGP Egress Traffic Engineering), with the guide's column padding removed; see the README
    // "Sources".
    const DOC_SRV6_TE: &str = r#"
protocols {
    source-packet-routing {
        segment-list end-sids-segment {
            srv6;
            hop1 srv6-sid 2001:db8:0:a4::d04;
            hop2 srv6-sid 2001:db8:0:a5::d05;
            hop3 srv6-sid 2001:db8:0:a6::d06;
        }
        segment-list end-x-sids-segment-last-sid-end-sid {
            srv6;
            hop1 srv6-sid 2001:db8:0:a2::1a23;
            hop2 srv6-sid 2001:db8:0:a3::1a34;
            hop3 srv6-sid 2001:db8:0:a6::d06;
        }
        srv6;
        source-routing-path nc_path_R1R6 {
            srv6;
            to 2001:db8:0:a6::d06;
            from 2001:db8:1:255::1;
            primary {
                end-sids-segment weight 40;
                end-x-sids-segment-last-sid-end-sid weight 30;
            }
        }
        source-routing-path c_path_R1R6 {
            srv6;
            to 2001:db8:0:a6::d06;
            from 2001:db8:1:255::1;
            color 6;
            primary {
                end-sids-segment weight 40;
                end-x-sids-segment-last-sid-end-sid weight 30;
            }
        }
    }
}
"#;

    // Not from documentation: written from the CLI Reference (`source-packet-routing`, `segment-list`,
    // `source-routing-path`). See the README "Sources".
    const EXTRA: &str = r#"
protocols {
    source-packet-routing {
        inherit-label-nexthops;
        lsp-external-controller PCE1;
        maximum-segment-list-depth 8;
        maximum-srv6-te-segment-list-depth 5;
        preference 8;
        preserve-nexthop-hierarchy;
        retry-timer 60;
        sbfd-frr;
        sr-preference 200;
        sr-preference-override 300;
        tunnel-tracking;
        segment-list MPLS-SL {
            auto-translate {
                protected mandatory;
            }
            compute;
            dynamic;
            inherit-label-nexthops;
            hop1 {
                ip-address 192.0.2.1;
                label 801001;
                label-type node;
                strict;
            }
            hop2 {
                label 801002;
                loose;
            }
        }
        source-routing-path MPLS-PATH {
            to 192.0.2.7;
            color 1200;
            binding-sid 1200;
            metric 500;
            no-ingress;
            sr-preference 70;
            te-group-id 5;
            lsp-external-controller PCE1;
            primary sr1 {
                weight 1;
                return-path sr1-return;
            }
            primary sr4 {
                compute PROFILE-A;
                weight 2;
            }
            secondary sr9 {
                weight 3;
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_guide_segment_lists() {
        let spr = source_packet_routing(&juniper::parse_braces(DOC_SRV6_TE)).unwrap();
        assert!(spr.srv6);
        assert_eq!(spr.segment_lists.len(), 2);
        let first = &spr.segment_lists[0];
        assert_eq!(first.name, "end-sids-segment");
        assert!(first.srv6);
        assert_eq!(first.hops.len(), 3);
        assert_eq!(first.hops[0].name, "hop1");
        assert_eq!(first.hops[0].srv6_sid.as_deref(), Some("2001:db8:0:a4::d04"));
        assert_eq!(first.hops[2].srv6_sid.as_deref(), Some("2001:db8:0:a6::d06"));
        assert_eq!(first.hops[0].label, None);
    }

    #[test]
    fn reads_the_guide_source_routing_paths() {
        let spr = source_packet_routing(&juniper::parse_braces(DOC_SRV6_TE)).unwrap();
        assert_eq!(spr.source_routing_paths.len(), 2);

        let uncolored = &spr.source_routing_paths[0];
        assert_eq!(uncolored.name, "nc_path_R1R6");
        assert!(uncolored.srv6);
        assert_eq!(uncolored.to.as_deref(), Some("2001:db8:0:a6::d06"));
        assert_eq!(uncolored.from.as_deref(), Some("2001:db8:1:255::1"));
        assert_eq!(uncolored.color, None);
        assert_eq!(uncolored.primary.len(), 2);
        assert_eq!(uncolored.primary[0].name, "end-sids-segment");
        assert_eq!(uncolored.primary[0].weight.as_deref(), Some("40"));
        assert_eq!(uncolored.primary[1].name, "end-x-sids-segment-last-sid-end-sid");
        assert_eq!(uncolored.primary[1].weight.as_deref(), Some("30"));

        assert_eq!(spr.source_routing_paths[1].color.as_deref(), Some("6"));
    }

    #[test]
    fn reads_the_global_options() {
        let spr = source_packet_routing(&juniper::parse_braces(EXTRA)).unwrap();
        assert!(spr.inherit_label_nexthops);
        assert_eq!(spr.lsp_external_controller.as_deref(), Some("PCE1"));
        assert_eq!(spr.maximum_segment_list_depth.as_deref(), Some("8"));
        assert_eq!(spr.maximum_srv6_te_segment_list_depth.as_deref(), Some("5"));
        assert_eq!(spr.preference.as_deref(), Some("8"));
        assert!(spr.preserve_nexthop_hierarchy);
        assert_eq!(spr.retry_timer.as_deref(), Some("60"));
        assert!(spr.sbfd_frr);
        assert_eq!(spr.sr_preference.as_deref(), Some("200"));
        assert_eq!(spr.sr_preference_override.as_deref(), Some("300"));
        assert!(spr.tunnel_tracking);
        assert!(!spr.srv6);
    }

    #[test]
    fn reads_hop_blocks_and_list_options() {
        let spr = source_packet_routing(&juniper::parse_braces(EXTRA)).unwrap();
        let list = &spr.segment_lists[0];
        assert!(list.auto_translate);
        assert!(list.compute);
        assert!(list.dynamic);
        assert!(list.inherit_label_nexthops);
        assert!(!list.srv6);
        assert_eq!(list.hops.len(), 2);

        let first = &list.hops[0];
        assert_eq!(first.ip_address.as_deref(), Some("192.0.2.1"));
        assert_eq!(first.label.as_deref(), Some("801001"));
        assert_eq!(first.label_type.as_deref(), Some("node"));
        assert!(first.strict);
        assert!(!first.loose);

        assert!(list.hops[1].loose);
        assert_eq!(list.hops[1].ip_address, None);
    }

    #[test]
    fn reads_path_options_and_named_primaries() {
        let spr = source_packet_routing(&juniper::parse_braces(EXTRA)).unwrap();
        let path = &spr.source_routing_paths[0];
        assert!(!path.srv6);
        assert_eq!(path.to.as_deref(), Some("192.0.2.7"));
        assert_eq!(path.binding_sid.as_deref(), Some("1200"));
        assert_eq!(path.metric.as_deref(), Some("500"));
        assert!(path.no_ingress);
        assert_eq!(path.sr_preference.as_deref(), Some("70"));
        assert_eq!(path.te_group_id.as_deref(), Some("5"));
        assert_eq!(path.lsp_external_controller.as_deref(), Some("PCE1"));

        assert_eq!(path.primary.len(), 2);
        assert_eq!(path.primary[0].name, "sr1");
        assert_eq!(path.primary[0].weight.as_deref(), Some("1"));
        assert_eq!(path.primary[0].return_path.as_deref(), Some("sr1-return"));
        assert_eq!(path.primary[1].compute.as_deref(), Some("PROFILE-A"));
        assert_eq!(path.secondary.len(), 1);
        assert_eq!(path.secondary[0].name, "sr9");
        assert_eq!(path.secondary[0].weight.as_deref(), Some("3"));
    }

    #[test]
    fn no_source_packet_routing_is_none() {
        let config = juniper::parse_braces("protocols {\n    ospf {\n        area 0.0.0.0;\n    }\n}\n");
        assert_eq!(source_packet_routing(&config), None);
    }
}
