//! `flex-algorithm`
//! `routing-options { flex-algorithm <id> { color <color>; definition { ... } } }`.
//!
//! # See also
//!
//! - [flex-algorithm](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/flex-algorithm-edit-routing-options.html)
//! - [definition](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/definition-edit-routing-options-flex-algorithm.html)
//! - [flex-algorithm (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/flex-algorithm-ospf-edit-routing-options.html)
//! - [definition (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/definition-ospf-edit-routing-options.html)
//! - [Configuring Flexible Algorithm for Segment Routing](https://networkingdocs.hpe.com/documentation/us/en/software/junos/is-is/segment-routing/topics/task/isis-configuring-flex-algo-sr.html)
//! - [Flexible Algorithms in IS-IS for Segment Routing Traffic Engineering](https://networkingdocs.hpe.com/documentation/us/en/software/junos/is-is/topics/topic-map/infocus-isis-flex-algo-sr.html)
//! - [How to Configure Flexible Algorithms in OSPF for Segment Routing Traffic Engineering](https://networkingdocs.hpe.com/documentation/us/en/software/junos/ospf/topics/topic-map/ospf-flex-algo-sr.html)

use crate::{active_children, arg_of, has_leaf, list_values, named_children, split_keyword, split_name};
use asterust::config::ConfigLine;

/// A `flex-algorithm <id>` of `routing-options`, a flexible algorithm definition (FAD) for IS-IS or OSPFv2.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FlexAlgorithm {
    /// 128 through 255.
    pub id: String,
    /// The BGP color associated with the algorithm.
    pub color: Option<String>,
    pub use_transport_class: bool,
    pub definition: Option<FlexAlgorithmDefinition>,
}

/// The `definition` of a [`FlexAlgorithm`]. The `exclude-srlg` and `use-flex-algorithm-prefix-metric` options are
/// only documented for IS-IS.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct FlexAlgorithmDefinition {
    pub spf: bool,
    pub strict_spf: bool,
    /// `delay-metric`, `igp-metric` or `te-metric`.
    pub metric_type: Option<String>,
    /// 0 through 255.
    pub priority: Option<String>,
    /// Admin group names from `admin-group include-any`.
    pub admin_group_include_any: Vec<String>,
    pub admin_group_include_all: Vec<String>,
    pub admin_group_exclude: Vec<String>,
    pub exclude_srlg: Vec<String>,
    pub use_flex_algorithm_prefix_metric: bool,
}

pub(super) fn parse_flex_algorithms(root: &ConfigLine) -> Vec<FlexAlgorithm> {
    named_children(root, "flex-algorithm")
        .map(|(arg, algorithm)| {
            let (id, rest) = split_name(arg);
            FlexAlgorithm {
                id: id.to_string(),
                color: arg_of(algorithm, "color").map(str::to_string),
                use_transport_class: rest.split_whitespace().any(|word| word == "use-transport-class")
                    || has_leaf(algorithm, "use-transport-class"),
                definition: parse_definition(algorithm),
            }
        })
        .collect()
}

/// Reads the `definition` block, including a collapsed `definition spf;` and a nested `admin-group { ... }`.
fn parse_definition(algorithm: &ConfigLine) -> Option<FlexAlgorithmDefinition> {
    let node =
        active_children(algorithm).find(|child| child.text == "definition" || child.arg("definition").is_some())?;
    let mut statements: Vec<String> = Vec::new();
    if let Some(inline) = node.arg("definition") {
        statements.push(inline.to_string());
    }
    for child in active_children(node) {
        let nested: Vec<_> = active_children(child).collect();
        if nested.is_empty() {
            statements.push(child.text.clone());
        } else {
            statements.extend(nested.iter().map(|inner| format!("{} {}", child.text, inner.text)));
        }
    }
    let mut definition = FlexAlgorithmDefinition::default();
    for statement in &statements {
        let (keyword, rest) = split_keyword(statement);
        let first = rest.split_whitespace().next().map(str::to_string);
        match keyword {
            "spf" => definition.spf = true,
            "strict-spf" => definition.strict_spf = true,
            "metric-type" => definition.metric_type = first,
            "priority" => definition.priority = first,
            "use-flex-algorithm-prefix-metric" => definition.use_flex_algorithm_prefix_metric = true,
            "exclude-srlg" => definition.exclude_srlg.extend(list_values(rest)),
            "admin-group" => {
                let (kind, names) = split_keyword(rest);
                let target = match kind {
                    "include-any" => &mut definition.admin_group_include_any,
                    "include-all" => &mut definition.admin_group_include_all,
                    "exclude" => &mut definition.admin_group_exclude,
                    _ => continue,
                };
                target.extend(list_values(names));
            }
            _ => {}
        }
    }
    Some(definition)
}

#[cfg(test)]
mod tests {
    use crate::routing_options::routing_options;
    use asterust::config::juniper;

    // Adapted from "How to Configure Flexible Algorithms in OSPF for Segment Routing Traffic Engineering" (OSPF User
    // Guide), Device R0 `show routing-options`, plus entries written from the CLI Reference; see the README "Sources".
    const CONFIG: &str = r#"
routing-options {
    router-id 192.168.255.10;
    flex-algorithm 128 {
        definition {
            metric-type igp-metric;
            spf;
            admin-group include-any RED;
        }
    }
    flex-algorithm 129 {
        definition {
            metric-type te-metric;
            spf;
            admin-group include-all [ GREEN BLUE ];
        }
    }
    flex-algorithm 130 {
        color 1300;
        use-transport-class;
        definition {
            strict-spf;
            metric-type delay-metric;
            priority 200;
            exclude-srlg [ SRLG-A SRLG-B ];
            use-flex-algorithm-prefix-metric;
            admin-group {
                exclude RED;
            }
        }
    }
    flex-algorithm 131 {
        definition;
    }
    flex-algorithm 132;
}
"#;

    #[test]
    fn reads_the_guide_definitions() {
        let options = routing_options(&juniper::parse_braces(CONFIG)).unwrap();
        assert_eq!(options.flex_algorithms.len(), 5);

        let first = &options.flex_algorithms[0];
        assert_eq!(first.id, "128");
        let definition = first.definition.as_ref().unwrap();
        assert_eq!(definition.metric_type.as_deref(), Some("igp-metric"));
        assert!(definition.spf);
        assert!(!definition.strict_spf);
        assert_eq!(definition.admin_group_include_any, ["RED"]);

        let second = options.flex_algorithms[1].definition.as_ref().unwrap();
        assert_eq!(second.metric_type.as_deref(), Some("te-metric"));
        assert_eq!(second.admin_group_include_all, ["GREEN", "BLUE"]);
        assert!(second.admin_group_include_any.is_empty());
    }

    #[test]
    fn reads_the_remaining_options() {
        let options = routing_options(&juniper::parse_braces(CONFIG)).unwrap();
        let algorithm = &options.flex_algorithms[2];
        assert_eq!(algorithm.color.as_deref(), Some("1300"));
        assert!(algorithm.use_transport_class);
        let definition = algorithm.definition.as_ref().unwrap();
        assert!(definition.strict_spf);
        assert_eq!(definition.metric_type.as_deref(), Some("delay-metric"));
        assert_eq!(definition.priority.as_deref(), Some("200"));
        assert_eq!(definition.exclude_srlg, ["SRLG-A", "SRLG-B"]);
        assert!(definition.use_flex_algorithm_prefix_metric);
        assert_eq!(definition.admin_group_exclude, ["RED"]);
    }

    #[test]
    fn reads_a_bare_definition_and_a_bare_algorithm() {
        let options = routing_options(&juniper::parse_braces(CONFIG)).unwrap();
        let bare_definition = &options.flex_algorithms[3];
        assert_eq!(bare_definition.id, "131");
        assert_eq!(bare_definition.definition, Some(Default::default()));

        let bare = &options.flex_algorithms[4];
        assert_eq!(bare.id, "132");
        assert_eq!(bare.definition, None);
        assert_eq!(bare.color, None);
    }
}
