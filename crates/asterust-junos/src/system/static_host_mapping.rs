//! `system { static-host-mapping { <hostname> { ... } } }`.

use crate::{active_children, arg_of, args_of, child_block, list_values, unquote};
use asterust::config::ConfigLine;

/// A `static-host-mapping <hostname>` entry.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct StaticHostMapping {
    pub hostname: String,
    pub aliases: Vec<String>,
    pub inet: Vec<String>,
    pub inet6: Vec<String>,
    /// The ISO system identifier, e.g. `0100.2550.0001`.
    pub sysid: Option<String>,
}

pub(super) fn static_host_mappings(system: &ConfigLine) -> Vec<StaticHostMapping> {
    child_block(system, "static-host-mapping")
        .map(|mappings| active_children(mappings).map(parse_static_host_mapping).collect())
        .unwrap_or_default()
}

fn parse_static_host_mapping(mapping: &ConfigLine) -> StaticHostMapping {
    let list = |keyword: &str| args_of(mapping, keyword).into_iter().flat_map(list_values).collect();
    StaticHostMapping {
        hostname: unquote(&mapping.text).to_string(),
        aliases: list("alias"),
        inet: list("inet"),
        inet6: list("inet6"),
        sysid: arg_of(mapping, "sysid").map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    #[test]
    fn extracts_static_host_mappings() {
        let system = system(&juniper::parse_braces(
            r#"
system {
    static-host-mapping {
        p1.example.net {
            inet [ 10.255.0.2 10.255.1.2 ];
            inet6 2001:db8::2;
            alias [ p1 core1 ];
            sysid 0100.2550.0002;
        }
        p2.example.net {
            inet 10.255.0.3;
        }
    }
}
"#,
        ))
        .unwrap();
        assert_eq!(system.static_host_mappings.len(), 2);

        let p1 = &system.static_host_mappings[0];
        assert_eq!(p1.hostname, "p1.example.net");
        assert_eq!(p1.inet, ["10.255.0.2", "10.255.1.2"]);
        assert_eq!(p1.inet6, ["2001:db8::2"]);
        assert_eq!(p1.aliases, ["p1", "core1"]);
        assert_eq!(p1.sysid.as_deref(), Some("0100.2550.0002"));

        let p2 = &system.static_host_mappings[1];
        assert_eq!(p2.inet, ["10.255.0.3"]);
        assert!(p2.aliases.is_empty());
        assert_eq!(p2.sysid, None);
    }
}
