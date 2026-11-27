//! `ospf`
//! `protocols { ospf { ... } }`.
//!
//! # See also
//!
//! - [ospf](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/ospf-edit-protocols.html)
//! - [area](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/area-edit-protocols-ospf.html)
//! - [interface (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/interface-edit-protocols-ospf.html)
//! - [reference-bandwidth (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/reference-bandwidth-edit-protocols-ospf.html)
//! - [export (OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/export-edit-protocols-ospf.html)
//! - [import (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/import-edit-protocols-ospf.html)
//! - [rib-group (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/rib-group-edit-protocols-ospf.html)
//! - [no-rfc-1583](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/no-rfc-1583-edit-protocols-ospf.html)
//! - [no-nssa-abr](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/no-nssa-abr-edit-protocols-ospf.html)
//! - [preference (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/preference-edit-protocols-ospf.html), from the Junos 12.3 archive
//! - [external-preference (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/external-preference-edit-protocols-ospf.html), from the Junos 12.3 archive

mod area;
mod authentication;
mod graceful_restart;
mod interface;
mod overload;
mod spf_options;

pub use area::{OspfArea, OspfAreaRange, OspfNssa, OspfStub, OspfVirtualLink};
pub use authentication::{OspfAuthentication, OspfMd5Key};
pub use interface::{OspfBfd, OspfInterface, OspfInterfaceTopology, OspfNeighbor, OspfPostConvergenceLfa};

pub use graceful_restart::OspfGracefulRestart;
pub use overload::OspfOverload;
pub use spf_options::OspfSpfOptions;

use crate::{arg_of, args_of, child_block, has_leaf, list_values, named_children, top_level};
use area::parse_area;
use asterust::config::Config;
use graceful_restart::parse_graceful_restart;
use overload::parse_overload;
use spf_options::parse_spf_options;

/// The `protocols ospf` configuration. `ospf3` is not covered.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Ospf {
    pub areas: Vec<OspfArea>,
    /// OSPF is disabled.
    pub disable: bool,
    /// Bits per second as written, e.g. `100g`.
    pub reference_bandwidth: Option<String>,
    /// Default 10.
    pub preference: Option<String>,
    /// For external routes; default 150.
    pub external_preference: Option<String>,
    pub export_policies: Vec<String>,
    pub import_policies: Vec<String>,
    pub rib_group: Option<String>,
    pub no_rfc_1583: bool,
    pub no_nssa_abr: bool,
    pub overload: Option<OspfOverload>,
    pub spf_options: Option<OspfSpfOptions>,
    pub graceful_restart: Option<OspfGracefulRestart>,
}

/// Returns the `protocols ospf` block, or `None` if OSPF is not configured.
///
/// # Examples
///
/// ```
/// use asterust::config::juniper;
/// use asterust_junos::protocols::ospf::ospf;
///
/// let config = juniper::parse_braces("protocols {\n    ospf {\n        area 0.0.0.0 {\n            interface lo0.0 {\n                passive;\n            }\n        }\n    }\n}\n");
/// let ospf = ospf(&config).unwrap();
/// assert_eq!(ospf.areas[0].id, "0.0.0.0");
/// assert!(ospf.areas[0].interfaces[0].passive);
/// ```
pub fn ospf(config: &Config) -> Option<Ospf> {
    let ospf = child_block(top_level(config, "protocols")?, "ospf")?;
    let areas = named_children(ospf, "area")
        .map(|(id, area)| parse_area(id, area))
        .collect();
    let policies =
        |keyword: &str| -> Vec<String> { args_of(ospf, keyword).into_iter().flat_map(list_values).collect() };
    Some(Ospf {
        areas,
        disable: has_leaf(ospf, "disable"),
        reference_bandwidth: arg_of(ospf, "reference-bandwidth").map(str::to_string),
        preference: arg_of(ospf, "preference").map(str::to_string),
        external_preference: arg_of(ospf, "external-preference").map(str::to_string),
        export_policies: policies("export"),
        import_policies: policies("import"),
        rib_group: arg_of(ospf, "rib-group").map(str::to_string),
        no_rfc_1583: has_leaf(ospf, "no-rfc-1583"),
        no_nssa_abr: has_leaf(ospf, "no-nssa-abr"),
        overload: parse_overload(ospf),
        spf_options: parse_spf_options(ospf),
        graceful_restart: parse_graceful_restart(ospf),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::juniper;

    // Not from documentation: covers statements the Juniper examples do not use. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    ospf {
        area 0.0.0.0 {
            interface ge-0/0/0.0 {
                metric 10;
            }
            interface lo0.0 {
                passive;
            }
        }
        area 0.0.0.10 {
            interface ge-0/0/1.0;
        }
    }
    ospf3 {
        area 0.0.0.0 {
            interface ge-0/0/0.0;
        }
    }
}
"#;

    #[test]
    fn extracts_areas_and_interfaces() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        assert_eq!(ospf.areas.len(), 2);

        let backbone = &ospf.areas[0];
        assert_eq!(backbone.id, "0.0.0.0");
        assert_eq!(backbone.interfaces.len(), 2);
        assert_eq!(backbone.interfaces[0].name, "ge-0/0/0.0");
        assert_eq!(backbone.interfaces[0].metric, Some(10));
        assert!(!backbone.interfaces[0].passive);
        assert!(backbone.interfaces[1].passive);

        let area10 = &ospf.areas[1];
        assert_eq!(area10.id, "0.0.0.10");
        assert_eq!(area10.interfaces[0].name, "ge-0/0/1.0");
    }

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const GLOBAL: &str = r#"
protocols {
    ospf {
        disable;
        reference-bandwidth 100g;
        preference 20;
        external-preference 160;
        export [ EXPORT-A EXPORT-B ];
        import IMPORT-A;
        rib-group RG-OSPF;
        no-rfc-1583;
        no-nssa-abr;
        area 0.0.0.0;
    }
}
"#;

    #[test]
    fn reads_the_global_options() {
        let ospf = ospf(&juniper::parse_braces(GLOBAL)).unwrap();
        assert!(ospf.disable);
        assert_eq!(ospf.reference_bandwidth.as_deref(), Some("100g"));
        assert_eq!(ospf.preference.as_deref(), Some("20"));
        assert_eq!(ospf.external_preference.as_deref(), Some("160"));
        assert_eq!(ospf.export_policies, ["EXPORT-A", "EXPORT-B"]);
        assert_eq!(ospf.import_policies, ["IMPORT-A"]);
        assert_eq!(ospf.rib_group.as_deref(), Some("RG-OSPF"));
        assert!(ospf.no_rfc_1583);
        assert!(ospf.no_nssa_abr);
    }

    #[test]
    fn leaves_the_global_options_unset_when_absent() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        assert!(!ospf.disable);
        assert_eq!(ospf.reference_bandwidth, None);
        assert!(ospf.export_policies.is_empty());
        assert_eq!(ospf.overload, None);
        assert_eq!(ospf.spf_options, None);
        assert_eq!(ospf.graceful_restart, None);
    }

    #[test]
    fn no_ospf_is_none() {
        assert!(ospf(&juniper::parse_braces("protocols {\n isis {\n }\n}\n")).is_none());
    }

    // Adapted from Juniper's BGP User Guide, "BGP Peering Sessions", "Example: Configuring Internal BGP Peer
    // Sessions", Device A `show protocols` (OSPF part); see the README "Sources".
    const DOC_OSPF: &str = r#"
protocols {
    ospf {
        area 0.0.0.0 {
            interface lo0.1 {
                passive;
            }
            interface ge-0/1/0.1;
        }
    }
}
"#;

    #[test]
    fn doc_area_with_passive_loopback() {
        let ospf = ospf(&juniper::parse_braces(DOC_OSPF)).unwrap();
        assert_eq!(ospf.areas.len(), 1);
        let area = &ospf.areas[0];
        assert_eq!(area.id, "0.0.0.0");
        assert_eq!(area.interfaces[0].name, "lo0.1");
        assert!(area.interfaces[0].passive);
        assert_eq!(area.interfaces[1].name, "ge-0/1/0.1");
        assert!(!area.interfaces[1].passive);
    }
}
