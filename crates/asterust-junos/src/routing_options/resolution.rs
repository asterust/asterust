//! `resolution`
//! `routing-options { resolution { rib <name> { ... } } }`.
//!
//! # See also
//!
//! - [resolution](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/resolution-edit-routing-options.html)
//! - [rib (Route Resolution)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/rib-edit-routing-options-resolution.html)

use crate::{args_of, has_leaf, list_values, named_children, toggle};
use asterust::config::ConfigLine;

/// A `rib <name>` of `resolution`: where next-hop resolution looks for routes to this table.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResolutionRib {
    pub name: String,
    pub import: Vec<String>,
    pub resolution_ribs: Vec<String>,
}

/// The `resolution` settings. `scheme`, `tracefilter` and `traceoptions` are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Resolution {
    pub ribs: Vec<ResolutionRib>,
    /// `Some(true)` for `allow-expanded-over-compressed`, `Some(false)` for the `no-` form.
    pub allow_expanded_over_compressed: Option<bool>,
    pub prefer_recursive_primary_path: Option<bool>,
    pub preserve_nexthop_hierarchy: Option<bool>,
    pub disable_auto_resolve_on_scheme_changes: bool,
    pub no_policy_metric_for_non_recursive_routes: bool,
}

pub(super) fn parse_resolution(resolution: &ConfigLine) -> Resolution {
    Resolution {
        ribs: named_children(resolution, "rib")
            .map(|(name, rib)| ResolutionRib {
                name: name.to_string(),
                import: args_of(rib, "import").into_iter().flat_map(list_values).collect(),
                resolution_ribs: args_of(rib, "resolution-ribs")
                    .into_iter()
                    .flat_map(list_values)
                    .collect(),
            })
            .collect(),
        allow_expanded_over_compressed: toggle(resolution, "allow-expanded-over-compressed"),
        prefer_recursive_primary_path: toggle(resolution, "prefer-recursive-primary-path"),
        preserve_nexthop_hierarchy: toggle(resolution, "preserve-nexthop-hierarchy"),
        disable_auto_resolve_on_scheme_changes: has_leaf(resolution, "disable-auto-resolve-on-scheme-changes"),
        no_policy_metric_for_non_recursive_routes: has_leaf(resolution, "no-policy-metric-for-non-recursive-routes"),
    }
}

#[cfg(test)]
mod tests {
    use crate::routing_options::routing_options;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_resolution_ribs_and_flags() {
        let options = routing_options(&juniper::parse_braces(
            r#"
routing-options {
    resolution {
        rib inet.3 {
            resolution-ribs [ inet.0 inet.3 ];
            import RESOLVE-POLICY;
        }
        rib inet6.3 {
            resolution-ribs inet6.0;
        }
        allow-expanded-over-compressed;
        no-prefer-recursive-primary-path;
        disable-auto-resolve-on-scheme-changes;
    }
}
"#,
        ))
        .unwrap();
        let resolution = options.resolution.unwrap();
        assert_eq!(resolution.ribs.len(), 2);
        assert_eq!(resolution.ribs[0].name, "inet.3");
        assert_eq!(resolution.ribs[0].resolution_ribs, ["inet.0", "inet.3"]);
        assert_eq!(resolution.ribs[0].import, ["RESOLVE-POLICY"]);
        assert_eq!(resolution.ribs[1].resolution_ribs, ["inet6.0"]);
        assert!(resolution.ribs[1].import.is_empty());
        assert_eq!(resolution.allow_expanded_over_compressed, Some(true));
        assert_eq!(resolution.prefer_recursive_primary_path, Some(false));
        assert_eq!(resolution.preserve_nexthop_hierarchy, None);
        assert!(resolution.disable_auto_resolve_on_scheme_changes);
        assert!(!resolution.no_policy_metric_for_non_recursive_routes);
    }

    #[test]
    fn no_resolution_is_none() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    router-id 192.0.2.1;\n}\n",
        ))
        .unwrap();
        assert!(options.resolution.is_none());
    }
}
