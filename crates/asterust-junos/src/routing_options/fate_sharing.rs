//! `fate-sharing`
//! `routing-options { fate-sharing { group <name> { ... } } }`.
//!
//! # See also
//!
//! - [fate-sharing](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/fate-sharing-edit-routing-options.html)

use crate::{active_children, arg_of, has_leaf, named_children, value_after};
use asterust::config::ConfigLine;

/// A `from <address> [to <address>]` line of a fate-sharing group.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FateSharingEntry {
    pub from: String,
    /// Set for a point-to-point link, which needs both ends.
    pub to: Option<String>,
}

/// A `group <name>` of objects that fail together.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FateSharingGroup {
    pub name: String,
    pub cost: Option<String>,
    pub entries: Vec<FateSharingEntry>,
    pub use_for_post_convergence_lfa: bool,
}

pub(super) fn parse_fate_sharing(fate_sharing: &ConfigLine) -> Vec<FateSharingGroup> {
    named_children(fate_sharing, "group")
        .map(|(name, group)| FateSharingGroup {
            name: name.to_string(),
            cost: arg_of(group, "cost").map(str::to_string),
            entries: active_children(group)
                .filter_map(|child| child.arg("from"))
                .map(|arg| {
                    let words: Vec<&str> = arg.split_whitespace().collect();
                    FateSharingEntry {
                        from: words.first().unwrap_or(&"").to_string(),
                        to: value_after(&words, "to"),
                    }
                })
                .collect(),
            use_for_post_convergence_lfa: has_leaf(group, "use-for-post-convergence-lfa"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::routing_options::routing_options;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_fate_sharing_groups() {
        let options = routing_options(&juniper::parse_braces(
            r#"
routing-options {
    fate-sharing {
        group LINK-A {
            cost 10;
            from 192.0.2.1 to 192.0.2.2;
            from 192.0.2.9;
            use-for-post-convergence-lfa;
        }
        group LINK-B {
            from 198.51.100.1;
        }
    }
}
"#,
        ))
        .unwrap();
        assert_eq!(options.fate_sharing.len(), 2);

        let a = &options.fate_sharing[0];
        assert_eq!(a.name, "LINK-A");
        assert_eq!(a.cost.as_deref(), Some("10"));
        assert_eq!(a.entries.len(), 2);
        assert_eq!(a.entries[0].from, "192.0.2.1");
        assert_eq!(a.entries[0].to.as_deref(), Some("192.0.2.2"));
        assert_eq!(a.entries[1].from, "192.0.2.9");
        assert_eq!(a.entries[1].to, None);
        assert!(a.use_for_post_convergence_lfa);

        let b = &options.fate_sharing[1];
        assert_eq!(b.cost, None);
        assert!(!b.use_for_post_convergence_lfa);
    }

    #[test]
    fn no_fate_sharing_is_empty() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    router-id 192.0.2.1;\n}\n",
        ))
        .unwrap();
        assert!(options.fate_sharing.is_empty());
    }
}
