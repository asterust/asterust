//! `active`, `install`, `metric`, `preference`, `tag`, `community` and the other route attributes
//! `routing-options { static { route <prefix> { ... } } }`; shared with `defaults` and, later, aggregate routes.
//!
//! # See also
//!
//! - [static (Routing Options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/static-edit-routing-options.html)
//! - [active](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/active-edit-routing-options.html)
//! - [install (Routing Options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/install-edit-routing-options.html)
//! - [readvertise](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/readvertise-edit-routing-options.html)
//! - [resolve](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/resolve-edit-routing-options.html)
//! - [retain](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/retain-edit-routing-options.html)
//! - [preference (Routing Options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/preference-edit-routing-options.html)
//! - [metric (Aggregate, Generated, or Static Route)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/metric-edit-routing-options.html)
//! - [tag (Routing Options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/tag-edit-routing-options.html)
//! - [community (Routing Options)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/community-edit-routing-options.html)

use crate::{list_values, split_keyword, unquote};

/// The attributes a route can carry, from the route itself or from the `defaults` of its block. `as-path`,
/// `bfd-liveness-detection` and the LSP next-hop statements are not extracted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct RouteAttributes {
    /// `Some(true)` for `active`, `Some(false)` for `passive`.
    pub active: Option<bool>,
    /// `Some(true)` for `install`, `Some(false)` for `no-install`.
    pub install: Option<bool>,
    pub readvertise: Option<bool>,
    pub resolve: Option<bool>,
    pub retain: Option<bool>,
    pub longest_match: Option<bool>,
    /// The value as written, with any `type` option, e.g. `"5"` or `"5 type 2"`.
    pub preference: Option<String>,
    pub preference2: Option<String>,
    pub metric: Option<String>,
    pub metric2: Option<String>,
    pub tag: Option<String>,
    pub tag2: Option<String>,
    pub color: Option<String>,
    pub color2: Option<String>,
    /// Community IDs and well-known names.
    pub community: Vec<String>,
    pub description: Option<String>,
    /// `high`, `medium` or `low`.
    pub priority: Option<String>,
    pub backup_pe_group: Option<String>,
}

/// Reads the attributes from statements such as `preference 5`, `no-readvertise` or `community [ a b ]`. Statements
/// that are not attributes are ignored.
pub(super) fn parse_route_attributes<'a>(statements: impl IntoIterator<Item = &'a str>) -> RouteAttributes {
    let mut attributes = RouteAttributes::default();
    for statement in statements {
        let (keyword, arg) = split_keyword(statement);
        let value = || Some(arg.to_string());
        match keyword {
            "active" => attributes.active = Some(true),
            "passive" => attributes.active = Some(false),
            "install" => attributes.install = Some(true),
            "no-install" => attributes.install = Some(false),
            "readvertise" => attributes.readvertise = Some(true),
            "no-readvertise" => attributes.readvertise = Some(false),
            "resolve" => attributes.resolve = Some(true),
            "no-resolve" => attributes.resolve = Some(false),
            "retain" => attributes.retain = Some(true),
            "no-retain" => attributes.retain = Some(false),
            "longest-match" => attributes.longest_match = Some(true),
            "no-longest-match" => attributes.longest_match = Some(false),
            "preference" => attributes.preference = value(),
            "preference2" => attributes.preference2 = value(),
            "metric" => attributes.metric = value(),
            "metric2" => attributes.metric2 = value(),
            "tag" => attributes.tag = value(),
            "tag2" => attributes.tag2 = value(),
            "color" => attributes.color = value(),
            "color2" => attributes.color2 = value(),
            "community" => attributes.community.extend(list_values(arg)),
            "description" => attributes.description = Some(unquote(arg).to_string()),
            "priority" => attributes.priority = value(),
            "backup-pe-group" => attributes.backup_pe_group = value(),
            _ => {}
        }
    }
    attributes
}

#[cfg(test)]
mod tests {
    use super::*;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn reads_flags_pairs_and_values() {
        let attributes = parse_route_attributes([
            "passive",
            "no-install",
            "readvertise",
            "no-resolve",
            "retain",
            "no-longest-match",
            "preference 5 type 2",
            "metric 3",
            "tag 7",
            "color 100",
            "community [ 65000:1 no-export ]",
            "description \"to branch\"",
            "priority high",
        ]);
        assert_eq!(attributes.active, Some(false));
        assert_eq!(attributes.install, Some(false));
        assert_eq!(attributes.readvertise, Some(true));
        assert_eq!(attributes.resolve, Some(false));
        assert_eq!(attributes.retain, Some(true));
        assert_eq!(attributes.longest_match, Some(false));
        assert_eq!(attributes.preference.as_deref(), Some("5 type 2"));
        assert_eq!(attributes.metric.as_deref(), Some("3"));
        assert_eq!(attributes.tag.as_deref(), Some("7"));
        assert_eq!(attributes.color.as_deref(), Some("100"));
        assert_eq!(attributes.community, ["65000:1", "no-export"]);
        assert_eq!(attributes.description.as_deref(), Some("to branch"));
        assert_eq!(attributes.priority.as_deref(), Some("high"));
    }

    #[test]
    fn ignores_statements_that_are_not_attributes() {
        let attributes = parse_route_attributes(["next-hop 192.0.2.1", "discard"]);
        assert_eq!(attributes, RouteAttributes::default());
    }
}
