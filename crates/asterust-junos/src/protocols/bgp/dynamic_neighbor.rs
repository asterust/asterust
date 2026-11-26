//! `dynamic-neighbor`, `allow` and `peer-as-list`
//! `protocols { bgp { group <name> { dynamic-neighbor <name> { ... } } } }`.
//!
//! # See also
//!
//! - [dynamic-neighbor](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/dynamic-neighbor-edit-protocols-bgp-group.html)
//! - [allow](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/allow-edit-protocols-bgp.html)
//! - [peer-as-list](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/peer-as-list-edit-protocols-bgp-group.html)

use crate::{arg_of, args_of, list_values};
use asterust::config::ConfigLine;

/// A `dynamic-neighbor <name>` block of a group, which accepts sessions from any address in its `allow` prefixes.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DynamicNeighbor {
    pub name: String,
    /// Prefixes written `allow [ 10.0.0.0/8 ];`, or `all`.
    pub allow: Vec<String>,
    pub peer_limit: Option<String>,
    pub authentication_algorithm: Option<String>,
    /// The name of the key chain; the key is not extracted.
    pub authentication_key_chain: Option<String>,
    /// An `authentication-key` is configured.
    pub has_authentication_key: bool,
}

/// Every `allow` entry of `line`, whether written as a list or as separate statements.
pub(super) fn parse_allow(line: &ConfigLine) -> Vec<String> {
    args_of(line, "allow").into_iter().flat_map(list_values).collect()
}

pub(super) fn parse_dynamic_neighbor(name: &str, block: &ConfigLine) -> DynamicNeighbor {
    DynamicNeighbor {
        name: name.to_string(),
        allow: parse_allow(block),
        peer_limit: arg_of(block, "peer-limit").map(str::to_string),
        authentication_algorithm: arg_of(block, "authentication-algorithm").map(str::to_string),
        authentication_key_chain: arg_of(block, "authentication-key-chain").map(str::to_string),
        has_authentication_key: arg_of(block, "authentication-key").is_some(),
    }
}

#[cfg(test)]
mod tests {
    use crate::protocols::bgp::bgp;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    bgp {
        group CUSTOMERS {
            type external;
            allow [ 192.0.2.0/24 198.51.100.0/24 ];
            allow 203.0.113.0/24;
            peer-as-list CUSTOMER-ASNS;
            dynamic-neighbor DYN {
                allow [ 10.0.0.0/8 ];
                peer-limit 100;
                authentication-algorithm md5;
                authentication-key "$9$secret";
            }
        }
    }
}
"#;

    #[test]
    fn reads_allow_peer_as_list_and_dynamic_neighbors() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let group = &bgp.groups[0];
        assert_eq!(group.allow, ["192.0.2.0/24", "198.51.100.0/24", "203.0.113.0/24"]);
        assert_eq!(group.peer_as_list.as_deref(), Some("CUSTOMER-ASNS"));

        let dynamic = &group.dynamic_neighbors[0];
        assert_eq!(dynamic.name, "DYN");
        assert_eq!(dynamic.allow, ["10.0.0.0/8"]);
        assert_eq!(dynamic.peer_limit.as_deref(), Some("100"));
        assert_eq!(dynamic.authentication_algorithm.as_deref(), Some("md5"));
        assert!(dynamic.has_authentication_key);
        assert_eq!(dynamic.authentication_key_chain, None);
    }
}
