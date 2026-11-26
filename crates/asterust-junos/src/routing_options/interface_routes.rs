//! `interface-routes`
//! `routing-options { interface-routes { family <family> { ... } rib-group <name>; } }`.
//!
//! # See also
//!
//! - [interface-routes](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/interface-routes-edit-routing-options.html)

use crate::{arg_of, child_block, has_leaf, named_children};
use asterust::config::ConfigLine;

/// A `family` of `interface-routes`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct InterfaceRoutesFamily {
    /// `inet` or `inet6`.
    pub name: String,
    /// `export { lan; }`.
    pub export_lan: bool,
    /// `export { point-to-point; }`.
    pub export_point_to_point: bool,
    pub import_primary: Option<String>,
}

/// The `interface-routes` settings that control which interface routes are installed in which tables.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct InterfaceRoutes {
    /// The `rib-group` argument as written, e.g. `"MY-GROUP"` or `"inet MY-GROUP"`.
    pub rib_group: Option<String>,
    pub families: Vec<InterfaceRoutesFamily>,
}

pub(super) fn parse_interface_routes(routes: &ConfigLine) -> InterfaceRoutes {
    InterfaceRoutes {
        rib_group: arg_of(routes, "rib-group").map(str::to_string),
        families: named_children(routes, "family")
            .map(|(name, family)| {
                let export = child_block(family, "export");
                InterfaceRoutesFamily {
                    name: name.to_string(),
                    export_lan: export.is_some_and(|export| has_leaf(export, "lan")),
                    export_point_to_point: export.is_some_and(|export| has_leaf(export, "point-to-point")),
                    import_primary: arg_of(family, "import-primary").map(str::to_string),
                }
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use crate::routing_options::routing_options;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_interface_routes() {
        let options = routing_options(&juniper::parse_braces(
            r#"
routing-options {
    interface-routes {
        rib-group inet IFL-GROUP;
        family inet {
            export {
                lan;
                point-to-point;
            }
            import-primary PRIMARY-ONLY;
        }
        family inet6 {
            export {
                lan;
            }
        }
    }
}
"#,
        ))
        .unwrap();
        let routes = options.interface_routes.unwrap();
        assert_eq!(routes.rib_group.as_deref(), Some("inet IFL-GROUP"));
        assert_eq!(routes.families.len(), 2);

        let inet = &routes.families[0];
        assert_eq!(inet.name, "inet");
        assert!(inet.export_lan);
        assert!(inet.export_point_to_point);
        assert_eq!(inet.import_primary.as_deref(), Some("PRIMARY-ONLY"));

        let inet6 = &routes.families[1];
        assert_eq!(inet6.name, "inet6");
        assert!(inet6.export_lan);
        assert!(!inet6.export_point_to_point);
        assert_eq!(inet6.import_primary, None);
    }

    #[test]
    fn no_interface_routes_is_none() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    router-id 192.0.2.1;\n}\n",
        ))
        .unwrap();
        assert!(options.interface_routes.is_none());
    }
}
