//! `traceoptions`
//! `protocols { <protocol> { traceoptions { file <name> [size <size>] [files <n>] [world-readable]; flag <flag> [<modifier>] [disable]; } } }`.
//!
//! The statement has the same shape in every protocol; only the protocol-specific flags differ.
//!
//! # See also
//!
//! - [traceoptions (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/traceoptions-edit-protocols-isis.html)

use crate::{active_children, has_leaf, named_children, split_name};
use asterust::config::ConfigLine;

/// A `traceoptions` block. Values are kept as written.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Traceoptions {
    /// The trace file name, without quotes.
    pub file: Option<String>,
    /// Maximum size of each trace file, e.g. `10m`.
    pub size: Option<String>,
    /// Maximum number of trace files.
    pub files: Option<String>,
    /// `Some(true)` for `world-readable`, `Some(false)` for `no-world-readable`.
    pub world_readable: Option<bool>,
    pub no_remote_trace: bool,
    pub flags: Vec<TraceFlag>,
}

/// A `flag <name> [<modifier>...] [disable]` of a [`Traceoptions`] block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TraceFlag {
    pub name: String,
    /// `detail`, `receive` and `send`, in the order written.
    pub modifiers: Vec<String>,
    pub disable: bool,
}

/// The first active `traceoptions` block of `parent`.
pub(crate) fn parse_traceoptions(parent: &ConfigLine) -> Option<Traceoptions> {
    let trace = active_children(parent).find(|child| child.text == "traceoptions")?;
    // `file` is read raw, since its name may be quoted and be followed by options.
    let file = active_children(trace).find_map(|child| child.arg("file"));
    let (name, options) = file.map_or((None, Vec::new()), |arg| {
        let (name, rest) = split_name(arg);
        (Some(name.to_string()), rest.split_whitespace().collect())
    });
    let option = |keyword: &str| {
        let position = options.iter().position(|word| *word == keyword)?;
        options.get(position + 1).map(|word| word.to_string())
    };
    let world_readable = if options.contains(&"world-readable") {
        Some(true)
    } else if options.contains(&"no-world-readable") {
        Some(false)
    } else {
        None
    };
    Some(Traceoptions {
        file: name,
        size: option("size"),
        files: option("files"),
        world_readable,
        no_remote_trace: has_leaf(trace, "no-remote-trace"),
        flags: named_children(trace, "flag")
            .map(|(arg, node)| {
                let mut words = arg
                    .split_whitespace()
                    .chain(active_children(node).flat_map(|child| child.text.split_whitespace()));
                let name = words.next().unwrap_or_default().to_string();
                let rest: Vec<&str> = words.collect();
                TraceFlag {
                    name,
                    disable: rest.contains(&"disable"),
                    modifiers: rest
                        .into_iter()
                        .filter(|word| matches!(*word, "detail" | "receive" | "send"))
                        .map(str::to_string)
                        .collect(),
                }
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use crate::protocols::isis::isis;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax page. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    isis {
        traceoptions {
            file "isis-log" size 10m files 5 world-readable;
            flag hello detail;
            flag lsp receive detail;
            flag spf disable;
            flag error;
        }
    }
}
"#;

    #[test]
    fn reads_the_file_options() {
        let trace = isis(&juniper::parse_braces(CONFIG)).unwrap().traceoptions.unwrap();
        assert_eq!(trace.file.as_deref(), Some("isis-log"));
        assert_eq!(trace.size.as_deref(), Some("10m"));
        assert_eq!(trace.files.as_deref(), Some("5"));
        assert_eq!(trace.world_readable, Some(true));
        assert!(!trace.no_remote_trace);
    }

    #[test]
    fn reads_the_flags_with_their_modifiers() {
        let trace = isis(&juniper::parse_braces(CONFIG)).unwrap().traceoptions.unwrap();
        let flags: Vec<_> = trace
            .flags
            .iter()
            .map(|flag| (flag.name.as_str(), flag.modifiers.clone(), flag.disable))
            .collect();
        assert_eq!(
            flags,
            [
                ("hello", vec!["detail".to_string()], false),
                ("lsp", vec!["receive".to_string(), "detail".to_string()], false),
                ("spf", vec![], true),
                ("error", vec![], false),
            ]
        );
    }

    #[test]
    fn reads_an_unquoted_file_and_no_world_readable() {
        let config = juniper::parse_braces(
            "protocols {\n    isis {\n        traceoptions {\n            file isis-log no-world-readable;\n            no-remote-trace;\n        }\n    }\n}\n",
        );
        let trace = isis(&config).unwrap().traceoptions.unwrap();
        assert_eq!(trace.file.as_deref(), Some("isis-log"));
        assert_eq!(trace.size, None);
        assert_eq!(trace.world_readable, Some(false));
        assert!(trace.no_remote_trace);
        assert!(trace.flags.is_empty());
    }

    #[test]
    fn is_none_when_absent() {
        let config = juniper::parse_braces("protocols {\n    isis {\n        interface ge-0/0/0.0;\n    }\n}\n");
        assert_eq!(isis(&config).unwrap().traceoptions, None);
    }
}
