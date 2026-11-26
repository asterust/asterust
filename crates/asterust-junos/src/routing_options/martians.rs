//! `martians`
//! `routing-options { martians { <prefix> <match-type> [allow]; } }`.
//!
//! # See also
//!
//! - [martians](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/martians-edit-routing-options.html)

use crate::{active_children, split_name};
use asterust::config::ConfigLine;

/// A martian entry: a prefix range that routes must not match, or `allow` to carve an exception out of one.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Martian {
    /// A prefix, or `default`.
    pub prefix: String,
    /// The match type as written, e.g. `"orlonger"`, `"upto /24"` or `"through 10.1.0.0/16"`.
    pub match_type: Option<String>,
    pub allow: bool,
}

/// Reads the entries of a `martians { ... }` block and any `martians <prefix> <match-type>;` lines.
pub(super) fn parse_martians(root: &ConfigLine) -> Vec<Martian> {
    let mut martians = Vec::new();
    for child in active_children(root) {
        if child.text == "martians" {
            martians.extend(active_children(child).map(|entry| parse_martian(&entry.text)));
        } else if let Some(entry) = child.arg("martians") {
            martians.push(parse_martian(entry));
        }
    }
    martians
}

/// `<prefix> [<match-type> ...] [allow]`.
fn parse_martian(entry: &str) -> Martian {
    let (prefix, rest) = split_name(entry);
    let (rest, allow) = match rest.strip_suffix("allow") {
        Some(before) if before.is_empty() || before.ends_with(' ') => (before.trim(), true),
        _ => (rest, false),
    };
    Martian {
        prefix: prefix.to_string(),
        match_type: Some(rest).filter(|m| !m.is_empty()).map(str::to_string),
        allow,
    }
}

#[cfg(test)]
mod tests {
    use crate::routing_options::routing_options;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_martian_block_entries() {
        let options = routing_options(&juniper::parse_braces(
            r#"
routing-options {
    martians {
        0.0.0.0/0 exact allow;
        10.0.0.0/8 orlonger;
        172.16.0.0/12 upto /24;
        192.168.0.0/16 through 192.168.255.0/24;
        default;
        inactive: 100.64.0.0/10 orlonger;
    }
}
"#,
        ))
        .unwrap();
        let martians = options.martians;
        assert_eq!(martians.len(), 5);
        assert_eq!(martians[0].prefix, "0.0.0.0/0");
        assert_eq!(martians[0].match_type.as_deref(), Some("exact"));
        assert!(martians[0].allow);
        assert_eq!(martians[1].match_type.as_deref(), Some("orlonger"));
        assert!(!martians[1].allow);
        assert_eq!(martians[2].match_type.as_deref(), Some("upto /24"));
        assert_eq!(martians[3].match_type.as_deref(), Some("through 192.168.255.0/24"));
        assert_eq!(martians[4].prefix, "default");
        assert_eq!(martians[4].match_type, None);
    }

    #[test]
    fn extracts_one_line_martians() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    martians 10.0.0.0/8 orlonger;\n    martians 10.1.0.0/16 exact allow;\n}\n",
        ))
        .unwrap();
        assert_eq!(options.martians.len(), 2);
        assert_eq!(options.martians[0].match_type.as_deref(), Some("orlonger"));
        assert!(options.martians[1].allow);
    }

    #[test]
    fn no_martians_is_empty() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    router-id 192.0.2.1;\n}\n",
        ))
        .unwrap();
        assert!(options.martians.is_empty());
    }
}
