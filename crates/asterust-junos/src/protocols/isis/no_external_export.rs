//! `no-external-export`
//! `protocols { isis { no-external-export protocol <bgp | isis | ospf | static>; } }`.
//!
//! # See also
//!
//! - [no-external-export](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/protocols-isis-no-external-export-protocol.html)

use crate::active_children;
use asterust::config::ConfigLine;

/// The protocols named by the `no-external-export` statements, whose routes are not exported into IS-IS. The
/// reference does not show the exact statement form, so the protocol names are accepted anywhere on the statement.
pub(super) fn parse_no_external_export(isis: &ConfigLine) -> Vec<String> {
    active_children(isis)
        .filter(|child| child.text == "no-external-export" || child.arg("no-external-export").is_some())
        .flat_map(|statement| {
            statement
                .text
                .split_whitespace()
                .chain(active_children(statement).flat_map(|child| child.text.split_whitespace()))
        })
        .filter(|word| matches!(*word, "bgp" | "isis" | "ospf" | "static"))
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::protocols::isis::isis;
    use asterust::config::juniper;

    // Not from documentation: the reference does not show this statement's display form. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    isis {
        no-external-export protocol bgp;
        no-external-export protocol static;
        no-external-export {
            protocol ospf;
        }
    }
}
"#;

    #[test]
    fn reads_the_named_protocols() {
        let isis = isis(&juniper::parse_braces(CONFIG)).unwrap();
        assert_eq!(isis.no_external_export, ["bgp", "static", "ospf"]);
    }

    #[test]
    fn is_empty_when_absent() {
        let config = juniper::parse_braces("protocols {\n    isis {\n        interface ge-0/0/0.0;\n    }\n}\n");
        assert!(isis(&config).unwrap().no_external_export.is_empty());
    }
}
