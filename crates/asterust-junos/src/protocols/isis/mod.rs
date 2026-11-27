//! `isis`
//! `protocols { isis { ... } }`.
//!
//! # See also
//!
//! - [isis](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/isis-edit-protocols.html)
//! - [interface (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/interface-edit-protocols-isis.html)
//! - [reference-bandwidth (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/reference-bandwidth-edit-protocols-isis.html)
//! - [export (IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/export-edit-protocols-isis.html)
//! - [import (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/import-edit-protocols-isis.html)
//! - [lsp-lifetime](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/lsp-lifetime-edit-protocols-isis.html)
//! - [max-areas](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/max-areas-edit-protocols-isis.html)
//! - [max-snp-size](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/max-snp-size-edit-protocols-isis.html)
//! - [no-ipv4-routing](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/no-ipv4-routing-edit-protocols-isis.html)
//! - [no-ipv6-routing](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/no-ipv6-routing-edit-protocols-isis.html)
//! - [loose-authentication-check](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/loose-authentication-check-edit-protocols-isis.html)
//! - [ignore-attached-bit](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/ignore-attached-bit-edit-protocols-isis.html)
//! - [rib-group (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/rib-group-edit-protocols-isis.html)
//! - [clns-routing](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/clns-routing-edit-protocols-isis.html)
//! - [clns-updown-compatibility](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/clns-updown-compatibility-edit-protocols-isis.html)
//! - [rfc7775-compliance](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/rfc7775-compliance-edit-protocols-isis.html)
//! - [strict-dual-isis (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/strict-dual-isis-edit-protocols-isis.html)
//! - [disable (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/disable-edit-protocols-isis.html), from the Junos 12.3 archive

mod graceful_restart;
mod interface;
mod level;
mod overload;
mod spf_options;

pub use graceful_restart::IsisGracefulRestart;
pub use interface::{IsisBfd, IsisInterface, IsisInterfaceLevel};
pub use level::IsisLevel;
pub use overload::IsisOverload;
pub use spf_options::IsisSpfOptions;

use crate::{
    arg_of, args_of, child_block, has_leaf, list_values, named_children, statement_words, top_level, value_after,
};
use asterust::config::Config;
use graceful_restart::parse_graceful_restart;
use interface::parse_interface;
use level::parse_level;
use overload::parse_overload;
use spf_options::parse_spf_options;

/// The `protocols isis` configuration. The NET lives on the loopback's `family iso` address, see
/// [`crate::interfaces::InterfaceUnit::iso_addresses`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Isis {
    pub interfaces: Vec<IsisInterface>,
    /// The `level <1|2>` entries directly under `isis`.
    pub levels: Vec<IsisLevel>,
    /// IS-IS is disabled.
    pub disable: bool,
    /// Bits per second as written.
    pub reference_bandwidth: Option<String>,
    pub export_policies: Vec<String>,
    pub import_policies: Vec<String>,
    /// Seconds.
    pub lsp_lifetime: Option<String>,
    pub max_areas: Option<String>,
    pub max_lsp_size: Option<String>,
    pub max_snp_size: Option<String>,
    pub no_ipv4_routing: bool,
    pub no_ipv6_routing: bool,
    pub no_authentication_check: bool,
    pub loose_authentication_check: bool,
    pub ignore_attached_bit: bool,
    pub clns_routing: bool,
    pub clns_updown_compatibility: bool,
    pub rfc7775_compliance: bool,
    /// The routing table group for `inet`, from `rib-group { inet ...; }`.
    pub rib_group_inet: Option<String>,
    pub rib_group_inet6: Option<String>,
    pub strict_dual_isis: bool,
    /// Seconds.
    pub strict_dual_isis_holddown: Option<String>,
    pub overload: Option<IsisOverload>,
    pub spf_options: Option<IsisSpfOptions>,
    pub graceful_restart: Option<IsisGracefulRestart>,
}

/// Returns the `protocols isis` block, or `None` if IS-IS is not configured.
///
/// # Examples
///
/// ```
/// use asterust::config::juniper;
/// use asterust_junos::protocols::isis::isis;
///
/// let config = juniper::parse_braces("protocols {\n    isis {\n        interface ge-0/0/0.0 {\n            point-to-point;\n        }\n    }\n}\n");
/// let isis = isis(&config).unwrap();
/// assert_eq!(isis.interfaces[0].name, "ge-0/0/0.0");
/// assert!(isis.interfaces[0].point_to_point);
/// ```
pub fn isis(config: &Config) -> Option<Isis> {
    let isis = child_block(top_level(config, "protocols")?, "isis")?;
    let policies =
        |keyword: &str| -> Vec<String> { args_of(isis, keyword).into_iter().flat_map(list_values).collect() };
    let rib_group = statement_words(isis, "rib-group");
    let strict_dual_isis = statement_words(isis, "strict-dual-isis");
    Some(Isis {
        interfaces: named_children(isis, "interface")
            .map(|(name, iface)| parse_interface(name, iface))
            .collect(),
        levels: named_children(isis, "level")
            .map(|(arg, level)| parse_level(arg, level))
            .collect(),
        disable: has_leaf(isis, "disable"),
        reference_bandwidth: arg_of(isis, "reference-bandwidth").map(str::to_string),
        export_policies: policies("export"),
        import_policies: policies("import"),
        lsp_lifetime: arg_of(isis, "lsp-lifetime").map(str::to_string),
        max_areas: arg_of(isis, "max-areas").map(str::to_string),
        max_lsp_size: arg_of(isis, "max-lsp-size").map(str::to_string),
        max_snp_size: arg_of(isis, "max-snp-size").map(str::to_string),
        no_ipv4_routing: has_leaf(isis, "no-ipv4-routing"),
        no_ipv6_routing: has_leaf(isis, "no-ipv6-routing"),
        no_authentication_check: has_leaf(isis, "no-authentication-check"),
        loose_authentication_check: has_leaf(isis, "loose-authentication-check"),
        ignore_attached_bit: has_leaf(isis, "ignore-attached-bit"),
        clns_routing: has_leaf(isis, "clns-routing"),
        clns_updown_compatibility: has_leaf(isis, "clns-updown-compatibility"),
        rfc7775_compliance: has_leaf(isis, "rfc7775-compliance"),
        rib_group_inet: rib_group.as_ref().and_then(|words| value_after(words, "inet")),
        rib_group_inet6: rib_group.as_ref().and_then(|words| value_after(words, "inet6")),
        strict_dual_isis: strict_dual_isis.is_some(),
        strict_dual_isis_holddown: strict_dual_isis.and_then(|words| value_after(&words, "holddown")),
        overload: parse_overload(isis),
        spf_options: parse_spf_options(isis),
        graceful_restart: parse_graceful_restart(isis),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::juniper;

    // Written from JunOS hierarchy knowledge, not adapted from documentation; see the README "Sources".
    const CONFIG: &str = r#"
protocols {
    isis {
        level 2 wide-metrics-only;
        interface ge-0/0/0.0 {
            point-to-point;
            level 1 disable;
        }
        interface lo0.0 {
            passive;
        }
        interface ge-0/0/1.0;
    }
}
"#;

    #[test]
    fn extracts_interfaces() {
        let isis = isis(&juniper::parse_braces(CONFIG)).unwrap();
        let names: Vec<_> = isis.interfaces.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, ["ge-0/0/0.0", "lo0.0", "ge-0/0/1.0"]);
        assert!(isis.interfaces[0].point_to_point);
        assert!(!isis.interfaces[0].passive);
        assert!(isis.interfaces[1].passive);
        assert!(!isis.interfaces[2].point_to_point);
    }

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const GLOBAL: &str = r#"
protocols {
    isis {
        disable;
        reference-bandwidth 100g;
        export [ EXPORT-A EXPORT-B ];
        import IMPORT-A;
        lsp-lifetime 1500;
        max-areas 5;
        max-lsp-size 1492;
        max-snp-size 1400;
        no-ipv4-routing;
        no-ipv6-routing;
        no-authentication-check;
        loose-authentication-check;
        ignore-attached-bit;
        clns-routing;
        clns-updown-compatibility;
        rfc7775-compliance;
        rib-group {
            inet RG-V4;
            inet6 RG-V6;
        }
        strict-dual-isis {
            holddown 30;
        }
    }
}
"#;

    #[test]
    fn reads_the_global_options() {
        let isis = isis(&juniper::parse_braces(GLOBAL)).unwrap();
        assert!(isis.disable);
        assert_eq!(isis.reference_bandwidth.as_deref(), Some("100g"));
        assert_eq!(isis.export_policies, ["EXPORT-A", "EXPORT-B"]);
        assert_eq!(isis.import_policies, ["IMPORT-A"]);
        assert_eq!(isis.lsp_lifetime.as_deref(), Some("1500"));
        assert_eq!(isis.max_areas.as_deref(), Some("5"));
        assert_eq!(isis.max_lsp_size.as_deref(), Some("1492"));
        assert_eq!(isis.max_snp_size.as_deref(), Some("1400"));
        assert!(isis.no_ipv4_routing);
        assert!(isis.no_ipv6_routing);
        assert!(isis.no_authentication_check);
        assert!(isis.loose_authentication_check);
        assert!(isis.ignore_attached_bit);
        assert!(isis.clns_routing);
        assert!(isis.clns_updown_compatibility);
        assert!(isis.rfc7775_compliance);
        assert_eq!(isis.rib_group_inet.as_deref(), Some("RG-V4"));
        assert_eq!(isis.rib_group_inet6.as_deref(), Some("RG-V6"));
        assert!(isis.strict_dual_isis);
        assert_eq!(isis.strict_dual_isis_holddown.as_deref(), Some("30"));
    }

    #[test]
    fn leaves_the_global_options_unset_when_absent() {
        let isis = isis(&juniper::parse_braces(CONFIG)).unwrap();
        assert!(!isis.disable);
        assert_eq!(isis.reference_bandwidth, None);
        assert!(isis.export_policies.is_empty());
        assert_eq!(isis.rib_group_inet, None);
        assert!(!isis.strict_dual_isis);
        assert_eq!(isis.overload, None);
        assert_eq!(isis.spf_options, None);
        assert_eq!(isis.graceful_restart, None);
    }

    #[test]
    fn no_isis_is_none() {
        assert!(isis(&juniper::parse_braces("protocols {\n bgp {\n }\n}\n")).is_none());
    }
}
