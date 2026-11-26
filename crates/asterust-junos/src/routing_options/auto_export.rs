//! `auto-export`
//! `routing-options { auto-export { family <family> { ... } } }`.
//!
//! # See also
//!
//! - [auto-export](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/auto-export-edit-routing-options.html)

use crate::{arg_of, child_block, has_leaf, named_children};
use asterust::config::ConfigLine;

/// The `unicast`, `multicast` or `flow` setting of an `auto-export` family.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AutoExportScope {
    pub disable: bool,
    pub rib_group: Option<String>,
}

/// A `family` of `auto-export`. `traceoptions` is not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AutoExportFamily {
    /// `inet`, `inet6` or `iso`.
    pub name: String,
    pub disable: bool,
    pub unicast: Option<AutoExportScope>,
    pub multicast: Option<AutoExportScope>,
    pub flow: Option<AutoExportScope>,
}

/// The `auto-export` settings for sharing routes between routing instances.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AutoExport {
    pub disable: bool,
    pub families: Vec<AutoExportFamily>,
}

pub(super) fn parse_auto_export(auto_export: &ConfigLine) -> AutoExport {
    AutoExport {
        disable: has_leaf(auto_export, "disable"),
        families: named_children(auto_export, "family")
            .map(|(name, family)| {
                let scope = |keyword: &str| {
                    child_block(family, keyword).map(|scope| AutoExportScope {
                        disable: has_leaf(scope, "disable"),
                        rib_group: arg_of(scope, "rib-group").map(str::to_string),
                    })
                };
                AutoExportFamily {
                    name: name.to_string(),
                    disable: has_leaf(family, "disable"),
                    unicast: scope("unicast"),
                    multicast: scope("multicast"),
                    flow: scope("flow"),
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
    fn extracts_auto_export_families() {
        let options = routing_options(&juniper::parse_braces(
            r#"
routing-options {
    auto-export {
        family inet {
            unicast {
                rib-group EXPORT-GROUP;
            }
            multicast {
                disable;
            }
        }
        family inet6 {
            disable;
        }
    }
}
"#,
        ))
        .unwrap();
        let auto_export = options.auto_export.unwrap();
        assert!(!auto_export.disable);
        assert_eq!(auto_export.families.len(), 2);

        let inet = &auto_export.families[0];
        assert_eq!(inet.name, "inet");
        assert!(!inet.disable);
        assert_eq!(
            inet.unicast.as_ref().unwrap().rib_group.as_deref(),
            Some("EXPORT-GROUP")
        );
        assert!(inet.multicast.as_ref().unwrap().disable);
        assert!(inet.flow.is_none());

        assert!(auto_export.families[1].disable);
    }

    #[test]
    fn bare_auto_export_enables_with_defaults() {
        let options = routing_options(&juniper::parse_braces("routing-options {\n    auto-export;\n}\n")).unwrap();
        let auto_export = options.auto_export.unwrap();
        assert!(!auto_export.disable);
        assert!(auto_export.families.is_empty());
    }

    #[test]
    fn no_auto_export_is_none() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    router-id 192.0.2.1;\n}\n",
        ))
        .unwrap();
        assert!(options.auto_export.is_none());
    }
}
