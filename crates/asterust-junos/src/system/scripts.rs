//! `scripts`
//! `system { scripts { ... } }`.
//!
//! # See also
//!
//! - [scripts](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/scripts-edit-system.html)

use crate::{arg_of, child_block, has_leaf, named_children};
use asterust::config::ConfigLine;

/// The `system scripts` configuration. Per-file options such as `source` and `checksum` are not
/// extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Scripts {
    /// File names from `commit { file <name>; }`.
    pub commit_files: Vec<String>,
    pub op_files: Vec<String>,
    pub event_files: Vec<String>,
    pub snmp_files: Vec<String>,
    /// e.g. `python3`.
    pub language: Option<String>,
    pub synchronize: bool,
    pub max_datasize: Option<String>,
}

pub(super) fn parse_scripts(scripts: &ConfigLine) -> Scripts {
    let files = |kind: &str| -> Vec<String> {
        child_block(scripts, kind)
            .map(|block| {
                named_children(block, "file")
                    .map(|(name, _)| name.to_string())
                    .collect()
            })
            .unwrap_or_default()
    };
    Scripts {
        commit_files: files("commit"),
        op_files: files("op"),
        event_files: files("event"),
        snmp_files: files("snmp"),
        language: arg_of(scripts, "language").map(str::to_string),
        synchronize: has_leaf(scripts, "synchronize"),
        max_datasize: arg_of(scripts, "max-datasize").map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax page. See the README "Sources".
    #[test]
    fn extracts_script_files_by_kind() {
        let scripts = system(&juniper::parse_braces(
            r#"
system {
    scripts {
        language python3;
        synchronize;
        max-datasize 128m;
        commit {
            file check-mtu.slax;
            file require-description.py {
                optional;
            }
        }
        op {
            file show-bgp.py;
        }
        event {
            file on-link-down.slax;
        }
        snmp {
            file oid.py;
        }
    }
}
"#,
        ))
        .unwrap()
        .scripts
        .unwrap();
        assert_eq!(scripts.commit_files, ["check-mtu.slax", "require-description.py"]);
        assert_eq!(scripts.op_files, ["show-bgp.py"]);
        assert_eq!(scripts.event_files, ["on-link-down.slax"]);
        assert_eq!(scripts.snmp_files, ["oid.py"]);
        assert_eq!(scripts.language.as_deref(), Some("python3"));
        assert!(scripts.synchronize);
        assert_eq!(scripts.max_datasize.as_deref(), Some("128m"));
    }

    #[test]
    fn scripts_default_to_empty() {
        let scripts = system(&juniper::parse_braces(
            "system {\n    scripts {\n        synchronize;\n    }\n}\n",
        ))
        .unwrap()
        .scripts
        .unwrap();
        assert!(scripts.commit_files.is_empty());
        assert!(scripts.op_files.is_empty());
        assert_eq!(scripts.language, None);
    }

    #[test]
    fn no_scripts_is_none() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.scripts.is_none());
    }
}
