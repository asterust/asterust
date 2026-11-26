//! `rib-groups`
//! `routing-options { rib-groups { <name> { import-rib [ ... ]; export-rib <table>; } } }`.
//!
//! # See also
//!
//! - [rib-groups](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/rib-groups-edit-routing-options.html)
//! - [import-rib](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/import-rib-edit-routing-options.html)
//! - [export-rib](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/export-rib-edit-routing-options.html)
//! - [import-policy](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/import-policy-edit-routing-options.html)

use crate::{active_children, arg_of, args_of, child_block, list_values};
use asterust::config::ConfigLine;

/// A routing-table group, which copies routes from one table into others.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct RibGroup {
    pub name: String,
    /// The tables that receive the routes; the first is the primary table.
    pub import_rib: Vec<String>,
    /// The table the routes come from, when it is not the primary import table.
    pub export_rib: Option<String>,
    pub import_policy: Vec<String>,
}

pub(super) fn parse_rib_groups(root: &ConfigLine) -> Vec<RibGroup> {
    child_block(root, "rib-groups")
        .map(|groups| {
            active_children(groups)
                .map(|group| RibGroup {
                    name: group.text.clone(),
                    import_rib: args_of(group, "import-rib").into_iter().flat_map(list_values).collect(),
                    export_rib: arg_of(group, "export-rib").map(str::to_string),
                    import_policy: args_of(group, "import-policy")
                        .into_iter()
                        .flat_map(list_values)
                        .collect(),
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use crate::routing_options::routing_options;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_rib_groups() {
        let options = routing_options(&juniper::parse_braces(
            r#"
routing-options {
    rib-groups {
        rib-interface-routes-v4 {
            import-rib [ inet.0 VRF.inet.0 ];
        }
        rib-import-vrf {
            export-rib VRF.inet.0;
            import-rib [ VRF.inet.0 inet.0 ];
            import-policy IMPORT-VRF;
        }
    }
}
"#,
        ))
        .unwrap();
        assert_eq!(options.rib_groups.len(), 2);

        let interface_routes = &options.rib_groups[0];
        assert_eq!(interface_routes.name, "rib-interface-routes-v4");
        assert_eq!(interface_routes.import_rib, ["inet.0", "VRF.inet.0"]);
        assert_eq!(interface_routes.export_rib, None);
        assert!(interface_routes.import_policy.is_empty());

        let vrf = &options.rib_groups[1];
        assert_eq!(vrf.export_rib.as_deref(), Some("VRF.inet.0"));
        assert_eq!(vrf.import_rib, ["VRF.inet.0", "inet.0"]);
        assert_eq!(vrf.import_policy, ["IMPORT-VRF"]);
    }

    #[test]
    fn no_rib_groups_is_empty() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    router-id 192.0.2.1;\n}\n",
        ))
        .unwrap();
        assert!(options.rib_groups.is_empty());
    }
}
