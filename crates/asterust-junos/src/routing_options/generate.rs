//! `generate`
//! `routing-options { generate { route <prefix> { ... } } }`.
//!
//! # See also
//!
//! - [generate](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/generate-edit-routing-options.html)
//! - [policy (Aggregate and Generated Routes)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/policy-edit-routing-options.html)
//! - [brief](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/brief-edit-routing-options.html)
//! - [full](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/full-edit-routing-options.html)

use super::route_attributes::{parse_route_attributes, route_statements, RouteAttributes};
use crate::{active_children, child_block, split_keyword, split_name, unquote};
use asterust::config::ConfigLine;

/// A generated `route <prefix>`. `as-path` is not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct GenerateRoute {
    pub prefix: String,
    /// The policy that selects the contributing routes.
    pub policy: Option<String>,
    pub brief: bool,
    pub full: bool,
    pub discard: bool,
    pub attributes: RouteAttributes,
}

/// The `generate` routes of a routing table.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct GenerateRoutes {
    /// The `defaults` attributes; `policy`, `brief`, `full` and `discard` inside `defaults` are not extracted.
    pub defaults: Option<RouteAttributes>,
    pub routes: Vec<GenerateRoute>,
}

pub(super) fn parse_generate_routes(generate: &ConfigLine) -> GenerateRoutes {
    GenerateRoutes {
        defaults: child_block(generate, "defaults")
            .map(|defaults| parse_route_attributes(active_children(defaults).map(|child| child.text.as_str()))),
        routes: active_children(generate)
            .filter_map(|line| Some(parse_route(line.arg("route")?, line)))
            .collect(),
    }
}

fn parse_route(arg: &str, line: &ConfigLine) -> GenerateRoute {
    let (prefix, inline) = split_name(arg);
    let statements = route_statements(inline, line);
    let mut route = GenerateRoute {
        prefix: prefix.to_string(),
        policy: None,
        brief: false,
        full: false,
        discard: false,
        attributes: RouteAttributes::default(),
    };
    for statement in &statements {
        let (keyword, arg) = split_keyword(statement);
        match keyword {
            "policy" => route.policy = Some(unquote(arg).to_string()),
            "brief" => route.brief = true,
            "full" => route.full = true,
            "discard" => route.discard = true,
            _ => {}
        }
    }
    route.attributes = parse_route_attributes(statements);
    route
}

#[cfg(test)]
mod tests {
    use crate::routing_options::routing_options;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_defaults_and_routes() {
        let options = routing_options(&juniper::parse_braces(
            r#"
routing-options {
    generate {
        defaults {
            preference 100;
        }
        route 0.0.0.0/0 {
            policy HAVE-UPSTREAM;
            metric 10;
            full;
        }
        route 10.0.0.0/8 discard;
    }
}
"#,
        ))
        .unwrap();
        let generate = options.generate.unwrap();
        assert_eq!(generate.defaults.unwrap().preference.as_deref(), Some("100"));
        assert_eq!(generate.routes.len(), 2);

        let default = &generate.routes[0];
        assert_eq!(default.prefix, "0.0.0.0/0");
        assert_eq!(default.policy.as_deref(), Some("HAVE-UPSTREAM"));
        assert!(default.full);
        assert!(!default.brief);
        assert_eq!(default.attributes.metric.as_deref(), Some("10"));
        assert!(generate.routes[1].discard);
    }

    #[test]
    fn no_generate_is_none() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    router-id 192.0.2.1;\n}\n",
        ))
        .unwrap();
        assert!(options.generate.is_none());
    }
}
