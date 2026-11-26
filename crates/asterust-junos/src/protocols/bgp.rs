//! Extracts the `bgp` settings of `protocols`.
//!
//! Configuration path: `protocols { bgp { ... } }`.
//!
//! # See also
//!
//! - [bgp](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/bgp-edit-protocols.html)
//! - [group (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/group-edit-protocols-bgp.html)
//! - [neighbor (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/neighbor-edit-protocols-bgp.html)

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

/// Returns the `protocols bgp` block, or `None` if BGP is not configured.
///
/// # Examples
///
/// ```
/// use asterust::config::juniper;
/// use asterust_junos::protocols::bgp::bgp;
///
/// let config = juniper::parse_braces("protocols {\n    bgp {\n        group IBGP {\n            type internal;\n        }\n    }\n}\n");
/// let bgp = bgp(&config).unwrap();
/// assert_eq!(bgp.groups[0].name, "IBGP");
/// assert_eq!(bgp.groups[0].group_type.as_deref(), Some("internal"));
/// ```
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

    // Not from documentation: covers statements the Juniper examples do not use. See the README "Sources".
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

    // The DOC_* fixtures are adapted from Juniper's BGP User Guide; see the README "Sources".

    // "BGP Peering Sessions", "Example: Configuring External BGP Point-to-Point Peer Sessions", Device E `show protocols`.
    const DOC_EXTERNAL_PEERS: &str = r#"
protocols {
    bgp {
        group external-peers {
            type external;
            peer-as 22;
            neighbor 10.10.10.2;
            neighbor 10.10.10.6;
            neighbor 10.10.10.10;
            neighbor 10.21.7.2 {
                peer-as 79;
            }
        }
    }
}
"#;

    // "BGP Peering Sessions", "Example: Configuring Internal BGP Peer Sessions", Device A `show protocols`
    // (the guide's typographic quotes around the description are written as plain quotes).
    const DOC_INTERNAL_PEERS: &str = r#"
protocols {
    bgp {
        group internal-peers {
            type internal;
            description "connections to B and C";
            local-address 192.168.6.5;
            export send-direct;
            neighbor 192.163.6.4;
            neighbor 192.168.40.4;
        }
    }
}
"#;

    // "BGP Route Reflectors", "Example: Configuring a Route Reflector", Device A `show protocols`
    // (the guide's rendering of this output drops a closing brace; it is restored here).
    const DOC_ROUTE_REFLECTOR: &str = r#"
protocols {
    bgp {
        group rr-clients {
            type internal;
            local-address 192.168.6.5;
            export send-ospf;
            cluster 192.168.6.5;
            neighbor 192.163.6.4;
            neighbor 192.168.40.4;
        }
        group non-clients {
            type internal;
            local-address 192.168.6.5;
            export send-ospf;
            neighbor 192.168.0.1;
            neighbor 192.168.5.5;
        }
    }
}
"#;

    // "BGP Peering Sessions", "Example: Configure Multiple Single-Hop EBGP Sessions on Different Links Using the
    // Same IPv6 Link-Local Address", R1 `show protocols`.
    const DOC_LINK_LOCAL: &str = r#"
protocols {
    bgp {
        group external {
            peer-as 65542;
            local-as 65541;
            neighbor "fe80::20%ge-0/0/1.1";
            neighbor "fe80::20%ge-0/0/1.2";
        }
    }
}
"#;

    #[test]
    fn doc_external_peers_with_neighbor_level_peer_as() {
        let bgp = bgp(&juniper::parse_braces(DOC_EXTERNAL_PEERS)).unwrap();
        let group = &bgp.groups[0];
        assert_eq!(group.name, "external-peers");
        assert_eq!(group.group_type.as_deref(), Some("external"));
        assert_eq!(group.peer_as.as_deref(), Some("22"));
        let addresses: Vec<_> = group.neighbors.iter().map(|n| n.address.as_str()).collect();
        assert_eq!(addresses, ["10.10.10.2", "10.10.10.6", "10.10.10.10", "10.21.7.2"]);
        assert_eq!(group.neighbors[0].peer_as, None);
        assert_eq!(group.neighbors[3].peer_as.as_deref(), Some("79"));
    }

    #[test]
    fn doc_internal_peers_with_local_address_and_export() {
        let bgp = bgp(&juniper::parse_braces(DOC_INTERNAL_PEERS)).unwrap();
        let group = &bgp.groups[0];
        assert_eq!(group.name, "internal-peers");
        assert_eq!(group.group_type.as_deref(), Some("internal"));
        assert_eq!(group.description.as_deref(), Some("connections to B and C"));
        assert_eq!(group.local_address.as_deref(), Some("192.168.6.5"));
        assert_eq!(group.export_policies, ["send-direct"]);
        assert_eq!(group.neighbors.len(), 2);
    }

    #[test]
    fn doc_route_reflector_cluster_marks_client_group() {
        let bgp = bgp(&juniper::parse_braces(DOC_ROUTE_REFLECTOR)).unwrap();
        assert_eq!(bgp.groups[0].name, "rr-clients");
        assert_eq!(bgp.groups[0].cluster.as_deref(), Some("192.168.6.5"));
        assert_eq!(bgp.groups[1].name, "non-clients");
        assert_eq!(bgp.groups[1].cluster, None);
        assert_eq!(bgp.groups[1].neighbors.len(), 2);
    }

    #[test]
    fn doc_quoted_link_local_neighbors() {
        let bgp = bgp(&juniper::parse_braces(DOC_LINK_LOCAL)).unwrap();
        let group = &bgp.groups[0];
        assert_eq!(group.peer_as.as_deref(), Some("65542"));
        assert_eq!(group.local_as.as_deref(), Some("65541"));
        let addresses: Vec<_> = group.neighbors.iter().map(|n| n.address.as_str()).collect();
        assert_eq!(addresses, ["fe80::20%ge-0/0/1.1", "fe80::20%ge-0/0/1.2"]);
    }
}
