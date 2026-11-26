//! `router bgp <asn>` blocks.

use crate::{child_arg, child_args, line_arg};
use asterust::config::{Config, ConfigBlock};
use std::collections::HashSet;

/// A BGP neighbor assembled from its `neighbor <address> ...` lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BgpNeighbor {
    pub address: String,
    pub remote_as: Option<String>,
    pub description: Option<String>,
    pub peer_group: Option<String>,
}

/// An IOS XE BGP process. `asn` is kept as text since it may be asdot (`"1.10"`).
/// Peer-group definitions are not listed as neighbors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BgpProcess {
    pub asn: String,
    pub router_id: Option<String>,
    pub address_families: Vec<String>,
    pub neighbors: Vec<BgpNeighbor>,
}

impl BgpProcess {
    /// Build from a top-level `router bgp <asn>` block, or `None` for any other block.
    pub fn from_block(block: &ConfigBlock) -> Option<Self> {
        let root = block.get_root_line();
        let asn = line_arg(&root.text, "router bgp")?.split_whitespace().next()?;
        let neighbor_lines = child_args(root, "neighbor");

        let peer_groups: HashSet<&str> = neighbor_lines
            .iter()
            .filter_map(|arg| {
                let mut words = arg.split_whitespace();
                let name = words.next()?;
                (words.next() == Some("peer-group") && words.next().is_none()).then_some(name)
            })
            .collect();

        let mut neighbors: Vec<BgpNeighbor> = Vec::new();
        for arg in neighbor_lines {
            let Some((address, setting)) = arg.split_once(' ') else {
                continue;
            };
            if peer_groups.contains(address) {
                continue;
            }
            let idx = neighbors.iter().position(|n| n.address == address).unwrap_or_else(|| {
                neighbors.push(BgpNeighbor {
                    address: address.to_string(),
                    remote_as: None,
                    description: None,
                    peer_group: None,
                });
                neighbors.len() - 1
            });
            let setting = setting.trim();
            if let Some(value) = line_arg(setting, "remote-as") {
                neighbors[idx].remote_as = Some(value.to_string());
            } else if let Some(value) = line_arg(setting, "description") {
                neighbors[idx].description = Some(value.to_string());
            } else if let Some(value) = line_arg(setting, "peer-group") {
                neighbors[idx].peer_group = Some(value.to_string());
            }
        }

        Some(Self {
            asn: asn.to_string(),
            router_id: child_arg(root, "bgp router-id").map(str::to_string),
            address_families: child_args(root, "address-family")
                .into_iter()
                .map(str::to_string)
                .collect(),
            neighbors,
        })
    }
}

/// Every `router bgp` block in `config`.
pub fn bgp_processes(config: &Config) -> Vec<BgpProcess> {
    config.blocks.iter().filter_map(BgpProcess::from_block).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::cisco;

    const CONFIG: &str = "
router bgp 65000
 bgp router-id 10.255.0.1
 bgp log-neighbor-changes
 neighbor RR-CLIENTS peer-group
 neighbor RR-CLIENTS remote-as 65000
 neighbor 10.255.0.2 remote-as 65000
 neighbor 10.255.0.2 description rr1
 neighbor 10.255.0.2 update-source Loopback0
 neighbor 10.255.0.3 peer-group RR-CLIENTS
 neighbor 192.0.2.1 remote-as 64512
 !
 address-family ipv4
  network 10.255.0.1 mask 255.255.255.255
  neighbor 10.255.0.2 activate
 exit-address-family
 !
 address-family vpnv4
  neighbor 10.255.0.2 activate
 exit-address-family
!
";

    #[test]
    fn assembles_neighbors_from_flat_lines() {
        let config = cisco::parse(CONFIG);
        let procs = bgp_processes(&config);
        assert_eq!(procs.len(), 1);

        let bgp = &procs[0];
        assert_eq!(bgp.asn, "65000");
        assert_eq!(bgp.router_id.as_deref(), Some("10.255.0.1"));
        assert_eq!(bgp.address_families, ["ipv4", "vpnv4"]);
        assert_eq!(
            bgp.neighbors,
            [
                BgpNeighbor {
                    address: "10.255.0.2".into(),
                    remote_as: Some("65000".into()),
                    description: Some("rr1".into()),
                    peer_group: None,
                },
                BgpNeighbor {
                    address: "10.255.0.3".into(),
                    remote_as: None,
                    description: None,
                    peer_group: Some("RR-CLIENTS".into()),
                },
                BgpNeighbor {
                    address: "192.0.2.1".into(),
                    remote_as: Some("64512".into()),
                    description: None,
                    peer_group: None,
                },
            ]
        );
    }

    #[test]
    fn non_bgp_block_is_none() {
        let config = cisco::parse("router ospf 1\n router-id 1.1.1.1\n");
        assert!(BgpProcess::from_block(&config.blocks[0]).is_none());
    }
}
