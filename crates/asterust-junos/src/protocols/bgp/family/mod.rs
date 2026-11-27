//! `family`
//! `protocols { bgp { [group <name> { [neighbor <address> {] family <afi> { <safi> { ... } } } } }`; valid at the `bgp`, `group` and `neighbor` levels.
//!
//! # See also
//!
//! - [family (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/family-edit-protocols-bgp.html)
//! - [labeled-unicast (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/labeled-unicast-edit-protocols-bgp.html)
//! - [loops (BGP Address Family)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/loops-edit-protocols-bgp-family.html)
//! - [aigp](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/aigp-edit-protocols-bgp.html)
//! - [damping (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/damping-edit-protocols-bgp.html)
//! - [resolve-vpn](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/resolve-vpn-edit-protocols-bgp.html)
//! - [no-install](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/no-install-edit-protocols.html)
//! - [no-validate](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/no-validate-edit-protocols-bgp.html)
//! - [rib-group (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/rib-group-edit-protocols-bgp.html)
//! - [protection (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/protection-edit-protocols-bgp.html)
//! - [strip-nexthop](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/strip-nexthop-edit-protocols-bgp.html)
//! - [auto-discovery-only](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/auto-discovery-only-edit-protocols-bgp.html)
//! - [defer-initial-multipath-build](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/defer-initial-multipath-build-edit-protocols-bgp-family-inet-unicast.html)
//! - [per-group-label](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/per-group-label-edit-protocols-bgp.html)
//! - [explicit-null (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/explicit-null-edit-protocols-bgp.html)
//! - [extended-nexthop](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/autogen-protocols-bgp-group-family-inet-unicast-extended-nexthop.html)
//! - [entropy-label (protocols bgp)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/entropy-label-edit-protocols-bgp-family-inet-labeled-unicast.html)
//! - [graceful-restart (Long-Lived for BGP Restarter)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/graceful-restart-long-lived-per-family-edit-protocols-bgp.html)
//! - [advertise-srv6-service](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/advertise-srv6-service-edit-protocols-bgp-family-inet-unicast.html)
//! - [accept-srv6-service](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/accept-srv6-service-edit-protocols-bgp-family-inet-unicast.html)
//! - [extended-nexthop-color](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/extended-nexthop-color-edit-protocols-bgp-family-inet.html)
//! - [extended-nexthop-tunnel](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/extended-nexthop-tunnel.html)
//! - [BGP Route Prioritization](https://networkingdocs.hpe.com/documentation/us/en/software/junos/bgp/topics/topic-map/bgp-route-prioritization.html)

mod add_path;
mod entropy_label;
mod graceful_restart;
mod prefix_limit;

pub use add_path::{AddPath, AddPathSend};
pub use entropy_label::EntropyLabel;
pub use graceful_restart::FamilyGracefulRestart;
pub use prefix_limit::PrefixLimit;

use crate::{active_children, arg_of, has_leaf, named_children, statement_words, value_after};
use add_path::parse_add_path;
use asterust::config::ConfigLine;
use entropy_label::parse_entropy_label;
use graceful_restart::parse_family_graceful_restart;
use prefix_limit::parse_prefix_limit;

const SAFIS: [&str; 7] = [
    "unicast",
    "multicast",
    "labeled-unicast",
    "any",
    "flow",
    "segment-routing-te",
    "signaling",
];

/// An address family of a peer, e.g. `family inet { unicast { ... } }`. Families are not inherited between the
/// `bgp`, `group` and `neighbor` levels, so none is resolved here. Only the more common per-family statements are
/// extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpFamily {
    /// `inet`, `inet6`, `inet-vpn`, `l2vpn`, `evpn`, `route-target`, ...
    pub afi: String,
    /// `unicast`, `labeled-unicast`, `signaling`, ...; `None` for a family written without one.
    pub safi: Option<String>,
    pub prefix_limit: Option<PrefixLimit>,
    pub accepted_prefix_limit: Option<PrefixLimit>,
    pub add_path: Option<AddPath>,
    pub loops: Option<String>,
    /// `Some(true)` for `aigp`, `Some(false)` for `aigp { disable; }`.
    pub aigp: Option<bool>,
    pub rib_group: Option<String>,
    /// The policy of `no-validate`.
    pub no_validate: Option<String>,
    /// Seconds, from `defer-initial-multipath-build { maximum-delay ...; }`.
    pub defer_initial_multipath_build_max_delay: Option<String>,
    pub damping: bool,
    pub resolve_vpn: bool,
    pub no_install: bool,
    pub protection: bool,
    pub strip_nexthop: bool,
    pub auto_discovery_only: bool,
    pub per_group_label: bool,
    pub explicit_null: bool,
    pub extended_nexthop: bool,
    /// Sends the SRv6 service SID with the routes of this family.
    pub advertise_srv6_service: bool,
    /// Accepts the SRv6 service SID of received routes.
    pub accept_srv6_service: bool,
    /// Resolves colored prefixes over `inetcolor.0` or `inet6color.0` for segment routing traffic engineering.
    pub extended_nexthop_color: bool,
    pub extended_nexthop_tunnel: bool,
    /// `expedited` or `priority <n>`, as written.
    pub output_queue_priority: Option<String>,
    pub route_refresh_priority: Option<String>,
    pub withdraw_priority: Option<String>,
    pub entropy_label: Option<EntropyLabel>,
    pub graceful_restart: Option<FamilyGracefulRestart>,
}

/// Reads the `family` statements of a `bgp`, `group` or `neighbor` block, one entry per address and sub-address family.
pub(crate) fn parse_families(line: &ConfigLine) -> Vec<BgpFamily> {
    let mut families = Vec::new();
    for (afi, block) in named_children(line, "family") {
        let safis: Vec<(&str, &ConfigLine)> = active_children(block)
            .filter_map(|child| {
                let name = child.text.split_whitespace().next()?;
                SAFIS.contains(&name).then_some((name, child))
            })
            .collect();
        if safis.is_empty() {
            families.push(parse_family(afi, None, block));
        }
        for (safi, node) in safis {
            families.push(parse_family(afi, Some(safi), node));
        }
    }
    families
}

fn parse_family(afi: &str, safi: Option<&str>, node: &ConfigLine) -> BgpFamily {
    let field = |keyword: &str| arg_of(node, keyword).map(str::to_string);
    let defer = statement_words(node, "defer-initial-multipath-build");
    BgpFamily {
        afi: afi.to_string(),
        safi: safi.map(str::to_string),
        prefix_limit: parse_prefix_limit(node, "prefix-limit"),
        accepted_prefix_limit: parse_prefix_limit(node, "accepted-prefix-limit"),
        add_path: parse_add_path(node),
        loops: field("loops"),
        aigp: statement_words(node, "aigp").map(|words| !words.contains(&"disable")),
        rib_group: field("rib-group"),
        no_validate: field("no-validate"),
        defer_initial_multipath_build_max_delay: defer.and_then(|words| value_after(&words, "maximum-delay")),
        damping: has_leaf(node, "damping"),
        resolve_vpn: has_leaf(node, "resolve-vpn"),
        no_install: has_leaf(node, "no-install"),
        protection: has_leaf(node, "protection"),
        strip_nexthop: has_leaf(node, "strip-nexthop"),
        auto_discovery_only: has_leaf(node, "auto-discovery-only"),
        per_group_label: has_leaf(node, "per-group-label"),
        explicit_null: statement_words(node, "explicit-null").is_some(),
        extended_nexthop: has_leaf(node, "extended-nexthop"),
        advertise_srv6_service: has_leaf(node, "advertise-srv6-service"),
        accept_srv6_service: has_leaf(node, "accept-srv6-service"),
        extended_nexthop_color: has_leaf(node, "extended-nexthop-color"),
        extended_nexthop_tunnel: has_leaf(node, "extended-nexthop-tunnel"),
        output_queue_priority: priority(node, "output-queue-priority"),
        route_refresh_priority: priority(node, "route-refresh-priority"),
        withdraw_priority: priority(node, "withdraw-priority"),
        entropy_label: parse_entropy_label(node),
        graceful_restart: parse_family_graceful_restart(node),
    }
}

/// `expedited` or `priority <n>`, whether written as a leaf or in a block.
fn priority(node: &ConfigLine, keyword: &str) -> Option<String> {
    statement_words(node, keyword).map(|words| words.join(" "))
}

#[cfg(test)]
mod tests {
    use crate::protocols::bgp::bgp;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    bgp {
        family inet {
            unicast;
        }
        group CORE {
            type internal;
            family inet {
                unicast {
                    loops 2;
                    rib-group RG-UNICAST;
                    aigp;
                    damping;
                    defer-initial-multipath-build {
                        maximum-delay 120;
                    }
                }
                labeled-unicast {
                    resolve-vpn;
                    per-group-label;
                    explicit-null {
                        connected-only;
                    }
                    aigp {
                        disable;
                    }
                }
            }
            family inet-vpn {
                unicast {
                    no-install;
                }
            }
            family l2vpn {
                signaling;
            }
            family route-target;
            neighbor 10.0.0.2 {
                family inet6 {
                    any;
                }
            }
        }
    }
}
"#;

    #[test]
    fn reads_one_entry_per_address_and_sub_address_family() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let global: Vec<_> = bgp
            .options
            .address_families
            .iter()
            .map(|f| (f.afi.as_str(), f.safi.as_deref()))
            .collect();
        assert_eq!(global, [("inet", Some("unicast"))]);

        let group = &bgp.groups[0];
        let names: Vec<_> = group
            .options
            .address_families
            .iter()
            .map(|f| (f.afi.as_str(), f.safi.as_deref()))
            .collect();
        assert_eq!(
            names,
            [
                ("inet", Some("unicast")),
                ("inet", Some("labeled-unicast")),
                ("inet-vpn", Some("unicast")),
                ("l2vpn", Some("signaling")),
                ("route-target", None),
            ]
        );
        let neighbor = &group.neighbors[0].options.address_families[0];
        assert_eq!(
            (neighbor.afi.as_str(), neighbor.safi.as_deref()),
            ("inet6", Some("any"))
        );
    }

    #[test]
    fn reads_the_statements_of_each_family() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let families = &bgp.groups[0].options.address_families;

        let unicast = &families[0];
        assert_eq!(unicast.loops.as_deref(), Some("2"));
        assert_eq!(unicast.rib_group.as_deref(), Some("RG-UNICAST"));
        assert_eq!(unicast.aigp, Some(true));
        assert!(unicast.damping);
        assert_eq!(unicast.defer_initial_multipath_build_max_delay.as_deref(), Some("120"));
        assert!(!unicast.resolve_vpn);

        let labeled = &families[1];
        assert!(labeled.resolve_vpn);
        assert!(labeled.per_group_label);
        assert!(labeled.explicit_null);
        assert_eq!(labeled.aigp, Some(false));

        assert!(families[2].no_install);
    }

    #[test]
    fn leaves_the_settings_unset_when_absent() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let global = &bgp.options.address_families[0];
        assert_eq!(global.prefix_limit, None);
        assert_eq!(global.add_path, None);
        assert_eq!(global.aigp, None);
        assert_eq!(global.loops, None);
    }
}
