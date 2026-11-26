//! `confederation`
//! `routing-options { confederation <as> members [ <as> ... ]; }`.
//!
//! # See also
//!
//! - [confederation](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/confederation-edit-routing-options.html)

use crate::{arg_of, list_values, split_name};
use asterust::config::ConfigLine;

/// A BGP confederation: the confederation AS and its member ASes.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Confederation {
    pub autonomous_system: String,
    pub members: Vec<String>,
}

pub(super) fn parse_confederation(root: &ConfigLine) -> Option<Confederation> {
    let (autonomous_system, rest) = split_name(arg_of(root, "confederation")?);
    Some(Confederation {
        autonomous_system: autonomous_system.to_string(),
        members: rest.strip_prefix("members").map(list_values).unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use crate::routing_options::routing_options;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_confederation_and_members() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    confederation 65001 members [ 65002 65003 ];\n}\n",
        ))
        .unwrap();
        let confederation = options.confederation.unwrap();
        assert_eq!(confederation.autonomous_system, "65001");
        assert_eq!(confederation.members, ["65002", "65003"]);
    }

    #[test]
    fn single_member_needs_no_brackets() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    confederation 65001 members 65002;\n}\n",
        ))
        .unwrap();
        assert_eq!(options.confederation.unwrap().members, ["65002"]);
    }

    #[test]
    fn no_confederation_is_none() {
        let options = routing_options(&juniper::parse_braces(
            "routing-options {\n    router-id 192.0.2.1;\n}\n",
        ))
        .unwrap();
        assert!(options.confederation.is_none());
    }
}
