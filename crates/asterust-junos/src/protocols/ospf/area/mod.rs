//! `area`
//! `protocols { ospf { area <id> { ... } } }`.
//!
//! # See also
//!
//! - [area](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/area-edit-protocols-ospf.html)
//! - [network-summary-export](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/network-summary-export-edit-protocols-ospf.html)
//! - [network-summary-import](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/network-summary-import-edit-protocols-ospf.html)
//! - [label-switched-path (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/label-switched-path-edit-protocols-ospf.html)
//! - [context-identifier (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/context-identifier-edit-protocols-ospf.html)

mod area_range;
mod nssa;
mod stub;
mod virtual_link;

pub use area_range::OspfAreaRange;
pub use nssa::OspfNssa;
pub use stub::OspfStub;
pub use virtual_link::OspfVirtualLink;

use super::interface::{parse_interface, OspfInterface};
use crate::{arg_of, args_of, list_values, named_children, split_name, value_after};
use area_range::parse_area_ranges;
use asterust::config::ConfigLine;
use nssa::parse_nssa;
use stub::parse_stub;
use virtual_link::parse_virtual_link;

/// An `area <id>` and its interfaces. A `stub` and an `nssa` are mutually exclusive on a device, but both are
/// read if both are written. The OSPFv3-only `inter-area-prefix-export` and `-import` are not covered.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfArea {
    pub id: String,
    pub interfaces: Vec<OspfInterface>,
    pub stub: Option<OspfStub>,
    pub nssa: Option<OspfNssa>,
    pub area_ranges: Vec<OspfAreaRange>,
    pub virtual_links: Vec<OspfVirtualLink>,
    pub network_summary_export: Vec<String>,
    pub network_summary_import: Vec<String>,
    /// The `label-switched-path` entries as `(name, metric)`.
    pub label_switched_paths: Vec<(String, Option<String>)>,
    pub context_identifier: Option<String>,
}

pub(super) fn parse_area(id: &str, area: &ConfigLine) -> OspfArea {
    let policies =
        |keyword: &str| -> Vec<String> { args_of(area, keyword).into_iter().flat_map(list_values).collect() };
    OspfArea {
        id: id.to_string(),
        interfaces: named_children(area, "interface")
            .map(|(name, iface)| parse_interface(name, iface))
            .collect(),
        stub: parse_stub(area),
        nssa: parse_nssa(area),
        area_ranges: parse_area_ranges(area),
        virtual_links: named_children(area, "virtual-link")
            .map(|(header, link)| parse_virtual_link(header, link))
            .collect(),
        network_summary_export: policies("network-summary-export"),
        network_summary_import: policies("network-summary-import"),
        label_switched_paths: named_children(area, "label-switched-path")
            .map(|(arg, _)| {
                let (name, rest) = split_name(arg);
                let words: Vec<&str> = rest.split_whitespace().collect();
                (name.to_string(), value_after(&words, "metric"))
            })
            .collect(),
        context_identifier: arg_of(area, "context-identifier").map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use crate::protocols::ospf::ospf;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    ospf {
        area 0.0.0.0 {
            network-summary-export [ EXPORT-A EXPORT-B ];
            network-summary-import IMPORT-A;
            label-switched-path TO-PE2 metric 5;
            context-identifier 192.0.2.99;
            interface ge-0/0/0.0;
        }
        area 0.0.0.1 {
            interface ge-0/0/1.0;
        }
    }
}
"#;

    #[test]
    fn reads_the_area_policies_paths_and_context_identifier() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let area = &ospf.areas[0];
        assert_eq!(area.network_summary_export, ["EXPORT-A", "EXPORT-B"]);
        assert_eq!(area.network_summary_import, ["IMPORT-A"]);
        assert_eq!(
            area.label_switched_paths,
            [("TO-PE2".to_string(), Some("5".to_string()))]
        );
        assert_eq!(area.context_identifier.as_deref(), Some("192.0.2.99"));
        assert_eq!(area.interfaces.len(), 1);
    }

    #[test]
    fn leaves_the_area_settings_empty_when_absent() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let area = &ospf.areas[1];
        assert_eq!(area.stub, None);
        assert_eq!(area.nssa, None);
        assert!(area.area_ranges.is_empty());
        assert!(area.virtual_links.is_empty());
        assert!(area.network_summary_export.is_empty());
        assert_eq!(area.context_identifier, None);
    }
}
