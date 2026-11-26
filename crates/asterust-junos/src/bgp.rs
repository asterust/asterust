//! `protocols { bgp { ... } }`.

use crate::{arg_of, args_of, child_block, first_word, list_values, named_children, top_level};
use asterust::config::{Config, ConfigLine};

/// A `neighbor <address>` in a BGP group. Settings inherited from the group are not resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpNeighbor {
    pub address: String,
    pub description: Option<String>,
    pub peer_as: Option<String>,
    pub local_address: Option<String>,
}

/// A `group <name>`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpGroup {
    pub name: String,
    /// `internal` or `external`.
    pub group_type: Option<String>,
    pub description: Option<String>,
    pub peer_as: Option<String>,
    pub local_as: Option<String>,
    pub local_address: Option<String>,
    /// Route-reflector `cluster` id.
    pub cluster: Option<String>,
    /// Address families, e.g. `"inet"`, `"inet-vpn"`.
    pub families: Vec<String>,
    pub import_policies: Vec<String>,
    pub export_policies: Vec<String>,
    pub neighbors: Vec<BgpNeighbor>,
}

/// The `protocols bgp` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Bgp {
    pub groups: Vec<BgpGroup>,
}

/// The `protocols bgp` block, or `None` if BGP is not configured.
pub fn bgp(config: &Config) -> Option<Bgp> {
    let bgp = child_block(top_level(config, "protocols")?, "bgp")?;
    Some(Bgp {
        groups: named_children(bgp, "group")
            .map(|(name, group)| parse_group(name, group))
            .collect(),
    })
}

fn parse_group(name: &str, group: &ConfigLine) -> BgpGroup {
    let policies =
        |keyword: &str| -> Vec<String> { args_of(group, keyword).into_iter().flat_map(list_values).collect() };
    BgpGroup {
        name: name.to_string(),
        group_type: arg_of(group, "type").map(str::to_string),
        description: arg_of(group, "description").map(str::to_string),
        peer_as: arg_of(group, "peer-as").and_then(first_word).map(str::to_string),
        local_as: arg_of(group, "local-as").and_then(first_word).map(str::to_string),
        local_address: arg_of(group, "local-address").map(str::to_string),
        cluster: arg_of(group, "cluster").map(str::to_string),
        families: named_children(group, "family").map(|(f, _)| f.to_string()).collect(),
        import_policies: policies("import"),
        export_policies: policies("export"),
        neighbors: named_children(group, "neighbor")
            .map(|(address, n)| BgpNeighbor {
                address: address.to_string(),
                description: arg_of(n, "description").map(str::to_string),
                peer_as: arg_of(n, "peer-as").and_then(first_word).map(str::to_string),
                local_address: arg_of(n, "local-address").map(str::to_string),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::juniper;

    // Written from JunOS hierarchy knowledge, not adapted from documentation; see the README "Sources".
    const CONFIG: &str = r#"
protocols {
    bgp {
        group IBGP {
            type internal;
            local-address 10.255.0.1;
            family inet {
                unicast;
            }
            family inet-vpn {
                unicast;
            }
            cluster 10.255.0.1;
            export [ NEXT-HOP-SELF EXPORT-LOOPBACK ];
            neighbor 10.255.0.2 {
                description rr1;
            }
            neighbor 10.255.0.3;
        }
        group EBGP {
            type external;
            peer-as 65001;
            import FROM-PEER;
            neighbor 192.0.2.2 {
                peer-as 65002;
            }
        }
    }
}
"#;

    #[test]
    fn extracts_internal_group_with_reflector_settings() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        assert_eq!(bgp.groups.len(), 2);

        let ibgp = &bgp.groups[0];
        assert_eq!(ibgp.name, "IBGP");
        assert_eq!(ibgp.group_type.as_deref(), Some("internal"));
        assert_eq!(ibgp.local_address.as_deref(), Some("10.255.0.1"));
        assert_eq!(ibgp.cluster.as_deref(), Some("10.255.0.1"));
        assert_eq!(ibgp.families, ["inet", "inet-vpn"]);
        assert_eq!(ibgp.export_policies, ["NEXT-HOP-SELF", "EXPORT-LOOPBACK"]);
        assert!(ibgp.import_policies.is_empty());

        let addresses: Vec<_> = ibgp.neighbors.iter().map(|n| n.address.as_str()).collect();
        assert_eq!(addresses, ["10.255.0.2", "10.255.0.3"]);
        assert_eq!(ibgp.neighbors[0].description.as_deref(), Some("rr1"));
        assert_eq!(ibgp.neighbors[1].description, None);
    }

    #[test]
    fn external_group_and_neighbor_override() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let ebgp = &bgp.groups[1];
        assert_eq!(ebgp.group_type.as_deref(), Some("external"));
        assert_eq!(ebgp.peer_as.as_deref(), Some("65001"));
        assert_eq!(ebgp.import_policies, ["FROM-PEER"]);
        assert_eq!(ebgp.neighbors[0].peer_as.as_deref(), Some("65002"));
    }

    #[test]
    fn no_bgp_is_none() {
        assert!(bgp(&juniper::parse_braces("protocols {\n ospf {\n }\n}\n")).is_none());
    }
}
