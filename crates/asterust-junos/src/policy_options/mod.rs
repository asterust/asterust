//! `policy-options`
//! `policy-options { ... }`.
//!
//! # See also
//!
//! - [`[edit policy-options]` hierarchy](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/topic-map/hierarchy-edit-policy-options.html)

mod as_path;
mod as_path_group;
mod community;
mod policy_statement;
mod prefix_list;

pub use as_path::AsPath;
pub use as_path_group::AsPathGroup;
pub use community::Community;
pub use policy_statement::{
    CommunityAction, CommunityOperation, PolicyAction, PolicyActions, PolicyConditions, PolicyStatement, PolicyTerm,
    RouteFilter,
};
pub use prefix_list::PrefixList;

use crate::{active_children, named_children, top_level};
use as_path::parse_as_path;
use as_path_group::parse_as_path_group;
use asterust::config::Config;
use community::parse_community;
use policy_statement::parse_policy;
use prefix_list::parse_prefix_list;

/// The top-level `policy-options` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PolicyOptions {
    pub policy_statements: Vec<PolicyStatement>,
    pub communities: Vec<Community>,
    pub prefix_lists: Vec<PrefixList>,
    pub as_paths: Vec<AsPath>,
    pub as_path_groups: Vec<AsPathGroup>,
}

/// Returns the top-level `policy-options` block, or `None` if there is none.
///
/// # Examples
///
/// ```
/// use asterust::config::juniper;
/// use asterust_junos::policy_options::policy_options;
///
/// let config = juniper::parse_braces("policy-options {\n    prefix-list CUSTOMERS {\n        192.0.2.0/24;\n    }\n}\n");
/// let options = policy_options(&config).unwrap();
/// assert_eq!(options.prefix_lists[0].name, "CUSTOMERS");
/// assert_eq!(options.prefix_lists[0].prefixes, ["192.0.2.0/24"]);
/// ```
pub fn policy_options(config: &Config) -> Option<PolicyOptions> {
    let root = top_level(config, "policy-options")?;
    Some(PolicyOptions {
        policy_statements: named_children(root, "policy-statement")
            .map(|(name, line)| parse_policy(name, line))
            .collect(),
        communities: active_children(root).filter_map(parse_community).collect(),
        prefix_lists: named_children(root, "prefix-list")
            .map(|(name, line)| parse_prefix_list(name, line))
            .collect(),
        as_paths: active_children(root).filter_map(parse_as_path).collect(),
        as_path_groups: named_children(root, "as-path-group")
            .map(|(name, line)| parse_as_path_group(name, line))
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::juniper;

    fn parse(text: &str) -> PolicyOptions {
        policy_options(&juniper::parse_braces(text)).expect("fixture has a policy-options block")
    }

    // Adapted from Juniper's BGP User Guide "Routing Policies for BGP Communities"; see the README "Sources".
    const DOC_COMMUNITY_TAG: &str = r#"
policy-options {
    policy-statement Edu-to-isis {
        term 1 {
            from {
                protocol bgp;
                community Edu;
            }
            then {
                metric 63;
                accept;
            }
        }
    }
    policy-statement send-isis-and-direct {
        term 1 {
            from {
                protocol [ isis direct ];
                route-filter 10.0.0.0/16 orlonger;
                route-filter 192.168.0.0/16 orlonger;
            }
            then accept;
        }
    }
    community Edu members 2:5;
}
"#;

    #[test]
    fn doc_term_with_from_block_and_metric() {
        let options = parse(DOC_COMMUNITY_TAG);
        assert_eq!(options.policy_statements.len(), 2);

        let policy = &options.policy_statements[0];
        assert_eq!(policy.name, "Edu-to-isis");
        let term = &policy.terms[0];
        assert_eq!(term.name.as_deref(), Some("1"));
        assert_eq!(term.from.protocols, ["bgp"]);
        assert_eq!(term.from.communities, ["Edu"]);
        assert_eq!(term.then.metric.as_deref(), Some("63"));
        assert_eq!(term.then.action, Some(PolicyAction::Accept));
    }

    #[test]
    fn doc_protocol_list_route_filters_and_leaf_then() {
        let options = parse(DOC_COMMUNITY_TAG);
        let term = &options.policy_statements[1].terms[0];
        assert_eq!(term.from.protocols, ["isis", "direct"]);
        assert_eq!(term.from.route_filters.len(), 2);
        assert_eq!(term.from.route_filters[0].prefix, "10.0.0.0/16");
        assert_eq!(term.from.route_filters[0].match_type.as_deref(), Some("orlonger"));
        assert_eq!(term.then.action, Some(PolicyAction::Accept));
    }

    #[test]
    fn doc_one_line_community() {
        let options = parse(DOC_COMMUNITY_TAG);
        assert_eq!(options.communities.len(), 1);
        assert_eq!(options.communities[0].name, "Edu");
        assert_eq!(options.communities[0].members, ["2:5"]);
        assert!(!options.communities[0].invert_match);
    }

    // Adapted from the same guide's "Configuring a Routing Policy That Removes BGP Communities" and "...Based on
    // the Number of BGP Communities" examples (Devices R1 and R2).
    const DOC_COMMUNITY_ACTIONS: &str = r#"
policy-options {
    policy-statement statics {
        from protocol static;
        then {
            community add 1;
            accept;
        }
    }
    policy-statement remove-communities {
        term 1 {
            from protocol bgp;
            then {
                community delete wild;
                accept;
            }
        }
        term 2 {
            then reject;
        }
    }
    community wild members *:*;
    community 1 members [ 2:1 2:2 2:3 ];
}
"#;

    #[test]
    fn doc_policy_without_terms_has_one_unnamed_term() {
        let options = parse(DOC_COMMUNITY_ACTIONS);
        let statics = &options.policy_statements[0];
        assert_eq!(statics.terms.len(), 1);
        let term = &statics.terms[0];
        assert_eq!(term.name, None);
        assert_eq!(term.from.protocols, ["static"]);
        assert_eq!(
            term.then.communities,
            [CommunityAction {
                operation: CommunityOperation::Add,
                community: "1".to_string()
            }]
        );
        assert_eq!(term.then.action, Some(PolicyAction::Accept));
    }

    #[test]
    fn doc_community_delete_and_final_reject_term() {
        let options = parse(DOC_COMMUNITY_ACTIONS);
        let remove = &options.policy_statements[1];
        assert_eq!(remove.terms.len(), 2);
        assert_eq!(
            remove.terms[0].then.communities[0].operation,
            CommunityOperation::Delete
        );
        assert_eq!(remove.terms[0].then.communities[0].community, "wild");
        assert_eq!(remove.terms[1].name.as_deref(), Some("2"));
        assert_eq!(remove.terms[1].then.action, Some(PolicyAction::Reject));
    }

    #[test]
    fn doc_community_members_list() {
        let options = parse(DOC_COMMUNITY_ACTIONS);
        assert_eq!(options.communities[0].members, ["*:*"]);
        assert_eq!(options.communities[1].name, "1");
        assert_eq!(options.communities[1].members, ["2:1", "2:2", "2:3"]);
    }

    // Not from documentation: covers statements the Juniper examples do not use. See the README "Sources".
    const CONFIG: &str = r#"
policy-options {
    prefix-list CUSTOMERS {
        192.0.2.0/24;
        2001:db8::/32;
    }
    prefix-list DYNAMIC {
        apply-path "protocols bgp group <*> neighbor <*>";
    }
    as-path FROM-AS65001 ".* 65001";
    as-path-group TRANSIT {
        as-path TIER1 "65100 .*";
        as-path TIER2 "65200 .*";
    }
    community BLOCK {
        invert-match;
        members [ no-export 65000:666 ];
    }
    community WILD members "* : *";
    policy-statement IMPORT {
        term FROM-CUSTOMER {
            from {
                family inet;
                neighbor 192.0.2.1;
                prefix-list CUSTOMERS;
                prefix-list-filter CUSTOMERS orlonger;
                as-path FROM-AS65001;
                route-filter 10.0.0.0/8 upto /24;
                policy SUBROUTINE;
            }
            then {
                local-preference 200;
                next-hop self;
                as-path-prepend "65000 65000";
                next policy;
            }
        }
        inactive: term DISABLED {
            then reject;
        }
        term SKIP {
            from protocol ospf;
            then next term;
        }
        then reject;
    }
}
"#;

    #[test]
    fn extracts_prefix_lists_and_as_paths() {
        let options = parse(CONFIG);
        assert_eq!(options.prefix_lists[0].name, "CUSTOMERS");
        assert_eq!(options.prefix_lists[0].prefixes, ["192.0.2.0/24", "2001:db8::/32"]);
        assert_eq!(options.prefix_lists[0].apply_path, None);
        assert!(options.prefix_lists[1].prefixes.is_empty());
        assert_eq!(
            options.prefix_lists[1].apply_path.as_deref(),
            Some("protocols bgp group <*> neighbor <*>")
        );

        assert_eq!(options.as_paths.len(), 1);
        assert_eq!(options.as_paths[0].name, "FROM-AS65001");
        assert_eq!(options.as_paths[0].regex, ".* 65001");
        let group = &options.as_path_groups[0];
        assert_eq!(group.name, "TRANSIT");
        let names: Vec<_> = group.as_paths.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["TIER1", "TIER2"]);
        assert_eq!(group.as_paths[1].regex, "65200 .*");
    }

    #[test]
    fn extracts_block_and_quoted_communities() {
        let options = parse(CONFIG);
        assert_eq!(options.communities.len(), 2);
        assert_eq!(options.communities[0].name, "BLOCK");
        assert_eq!(options.communities[0].members, ["no-export", "65000:666"]);
        assert!(options.communities[0].invert_match);
        assert_eq!(options.communities[1].members, ["* : *"]);
    }

    #[test]
    fn extracts_conditions_and_manipulation_actions() {
        let options = parse(CONFIG);
        let term = &options.policy_statements[0].terms[0];
        assert_eq!(term.name.as_deref(), Some("FROM-CUSTOMER"));
        assert_eq!(term.from.families, ["inet"]);
        assert_eq!(term.from.neighbors, ["192.0.2.1"]);
        assert_eq!(term.from.prefix_lists, ["CUSTOMERS"]);
        assert_eq!(term.from.prefix_list_filters, ["CUSTOMERS orlonger"]);
        assert_eq!(term.from.as_paths, ["FROM-AS65001"]);
        assert_eq!(term.from.route_filters[0].match_type.as_deref(), Some("upto /24"));
        assert_eq!(term.from.policies, ["SUBROUTINE"]);
        assert_eq!(term.then.local_preference.as_deref(), Some("200"));
        assert_eq!(term.then.next_hop.as_deref(), Some("self"));
        assert_eq!(term.then.as_path_prepend.as_deref(), Some("65000 65000"));
        assert_eq!(term.then.action, Some(PolicyAction::NextPolicy));
    }

    #[test]
    fn skips_inactive_terms_and_keeps_unnamed_final_term_last() {
        let options = parse(CONFIG);
        let terms = &options.policy_statements[0].terms;
        let names: Vec<_> = terms.iter().map(|t| t.name.as_deref()).collect();
        assert_eq!(names, [Some("FROM-CUSTOMER"), Some("SKIP"), None]);
        assert_eq!(terms[1].then.action, Some(PolicyAction::NextTerm));
        assert_eq!(terms[2].then.action, Some(PolicyAction::Reject));
    }

    #[test]
    fn no_policy_options_is_none() {
        assert!(policy_options(&juniper::parse_braces("system {\n host-name r1;\n}\n")).is_none());
    }
}
