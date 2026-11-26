//! `router bgp <asn>` blocks.

use crate::{child_arg, child_args, line_arg};
use asterust::config::{Config, ConfigBlock};

/// A BGP neighbor configured directly under the process (not in a VRF).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BgpNeighbor {
    pub address: String,
    pub remote_as: Option<String>,
    pub description: Option<String>,
}

/// An IOS XR BGP process. `asn` is kept as text since it may be asdot (`"1.10"`).
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
        let neighbors = root
            .children
            .iter()
            .filter_map(|c| {
                let address = line_arg(&c.text, "neighbor")?;
                Some(BgpNeighbor {
                    address: address.to_string(),
                    remote_as: child_arg(c, "remote-as").map(str::to_string),
                    description: child_arg(c, "description").map(str::to_string),
                })
            })
            .collect();
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
 address-family ipv4 unicast
  network 10.255.0.1/32
 !
 address-family vpnv4 unicast
 !
 neighbor-group RR-CLIENTS
  remote-as 65000
 !
 neighbor 10.255.0.2
  remote-as 65000
  description rr1
  address-family ipv4 unicast
   route-reflector-client
  !
 !
 neighbor 192.0.2.1
  remote-as 64512
 !
!
";

    #[test]
    fn extracts_process_families_and_neighbors() {
        let config = cisco::parse(CONFIG);
        let procs = bgp_processes(&config);
        assert_eq!(procs.len(), 1);

        let bgp = &procs[0];
        assert_eq!(bgp.asn, "65000");
        assert_eq!(bgp.router_id.as_deref(), Some("10.255.0.1"));
        assert_eq!(bgp.address_families, ["ipv4 unicast", "vpnv4 unicast"]);
        assert_eq!(
            bgp.neighbors,
            [
                BgpNeighbor {
                    address: "10.255.0.2".into(),
                    remote_as: Some("65000".into()),
                    description: Some("rr1".into()),
                },
                BgpNeighbor {
                    address: "192.0.2.1".into(),
                    remote_as: Some("64512".into()),
                    description: None,
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
