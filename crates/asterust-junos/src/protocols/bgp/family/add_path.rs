//! `add-path`
//! `protocols { bgp { group <name> { family <afi> { <safi> { add-path { receive; send { ... } } } } } } }`.
//!
//! # See also
//!
//! - [add-path](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/add-path-edit-protocols-bgp.html)
//! - [send (add-path)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/send-edit-protocols-bgp.html)

use crate::{args_of, child_block, has_leaf, list_values, statement_words, value_after};
use asterust::config::ConfigLine;

/// The `add-path` statement of an address family.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AddPath {
    pub receive: bool,
    pub send: Option<AddPathSend>,
}

/// The `send` block of `add-path`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct AddPathSend {
    pub path_count: Option<String>,
    pub include_backup_path: Option<String>,
    pub multipath: bool,
    /// `all-paths` or `equal-cost-paths`.
    pub path_selection_mode: Option<String>,
    pub prefix_policies: Vec<String>,
}

pub(super) fn parse_add_path(node: &ConfigLine) -> Option<AddPath> {
    let add_path = child_block(node, "add-path")?;
    let send = child_block(add_path, "send").map(|send| {
        let words = statement_words(add_path, "send").unwrap_or_default();
        AddPathSend {
            path_count: value_after(&words, "path-count"),
            include_backup_path: value_after(&words, "include-backup-path"),
            multipath: words.contains(&"multipath"),
            path_selection_mode: statement_words(send, "path-selection-mode")
                .and_then(|words| words.first().map(|word| word.to_string())),
            prefix_policies: args_of(send, "prefix-policy")
                .into_iter()
                .flat_map(list_values)
                .collect(),
        }
    });
    Some(AddPath {
        receive: has_leaf(add_path, "receive"),
        send,
    })
}

#[cfg(test)]
mod tests {
    use crate::protocols::bgp::bgp;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    bgp {
        group RR {
            family inet {
                unicast {
                    add-path {
                        receive;
                        send {
                            path-count 4;
                            include-backup-path 2;
                            path-selection-mode {
                                equal-cost-paths;
                            }
                            prefix-policy [ ADD-PATH-A ADD-PATH-B ];
                        }
                    }
                }
            }
            neighbor 10.0.0.2 {
                family inet {
                    unicast {
                        add-path {
                            send {
                                multipath;
                            }
                        }
                    }
                }
            }
        }
    }
}
"#;

    #[test]
    fn reads_receive_and_the_send_block() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let add_path = bgp.groups[0].options.address_families[0].add_path.as_ref().unwrap();
        assert!(add_path.receive);
        let send = add_path.send.as_ref().unwrap();
        assert_eq!(send.path_count.as_deref(), Some("4"));
        assert_eq!(send.include_backup_path.as_deref(), Some("2"));
        assert_eq!(send.path_selection_mode.as_deref(), Some("equal-cost-paths"));
        assert_eq!(send.prefix_policies, ["ADD-PATH-A", "ADD-PATH-B"]);
        assert!(!send.multipath);
    }

    #[test]
    fn reads_a_send_only_neighbor() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let add_path = bgp.groups[0].neighbors[0].options.address_families[0]
            .add_path
            .as_ref()
            .unwrap();
        assert!(!add_path.receive);
        assert!(add_path.send.as_ref().unwrap().multipath);
    }
}
