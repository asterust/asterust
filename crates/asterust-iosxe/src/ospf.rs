//! `router ospf <process> [vrf <name>]` blocks.

use crate::{child_arg, child_args, line_arg};
use asterust::config::{Config, ConfigBlock};

/// A `network <prefix> <wildcard> area <area>` statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OspfNetwork {
    pub prefix: String,
    pub wildcard: String,
    pub area: String,
}

/// An IOS XE OSPF process. Interfaces enabled with `ip ospf <process> area <area>`
/// are on [`crate::interface::Interface`], not here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OspfProcess {
    pub process_id: String,
    pub vrf: Option<String>,
    pub router_id: Option<String>,
    pub networks: Vec<OspfNetwork>,
    pub passive_interfaces: Vec<String>,
}

impl OspfProcess {
    /// Build from a top-level `router ospf` block, or `None` for any other block.
    pub fn from_block(block: &ConfigBlock) -> Option<Self> {
        let root = block.get_root_line();
        let mut words = line_arg(&root.text, "router ospf")?.split_whitespace();
        let process_id = words.next()?;
        let vrf = match (words.next(), words.next()) {
            (Some("vrf"), Some(name)) => Some(name.to_string()),
            _ => None,
        };
        Some(Self {
            process_id: process_id.to_string(),
            vrf,
            router_id: child_arg(root, "router-id").map(str::to_string),
            networks: child_args(root, "network")
                .into_iter()
                .filter_map(parse_network)
                .collect(),
            passive_interfaces: child_args(root, "passive-interface")
                .into_iter()
                .map(str::to_string)
                .collect(),
        })
    }
}

fn parse_network(arg: &str) -> Option<OspfNetwork> {
    let mut words = arg.split_whitespace();
    let prefix = words.next()?;
    let wildcard = words.next()?;
    if words.next()? != "area" {
        return None;
    }
    let area = words.next()?;
    Some(OspfNetwork {
        prefix: prefix.to_string(),
        wildcard: wildcard.to_string(),
        area: area.to_string(),
    })
}

/// Every `router ospf` block in `config`.
pub fn ospf_processes(config: &Config) -> Vec<OspfProcess> {
    config.blocks.iter().filter_map(OspfProcess::from_block).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::cisco;

    const CONFIG: &str = "
router ospf 1
 router-id 10.255.0.1
 passive-interface default
 no passive-interface GigabitEthernet0/0/0
 network 10.0.12.0 0.0.0.3 area 0
 network 10.1.0.0 0.0.255.255 area 10
!
router ospf 2 vrf CUSTOMER-A
 router-id 192.0.2.1
 passive-interface Loopback10
!
router ospfv3 3
 router-id 10.255.0.1
!
";

    #[test]
    fn extracts_networks_and_passive_interfaces() {
        let config = cisco::parse(CONFIG);
        let procs = ospf_processes(&config);
        assert_eq!(procs.len(), 2);

        let ospf = &procs[0];
        assert_eq!(ospf.process_id, "1");
        assert_eq!(ospf.vrf, None);
        assert_eq!(ospf.router_id.as_deref(), Some("10.255.0.1"));
        assert_eq!(ospf.passive_interfaces, ["default"]);
        assert_eq!(
            ospf.networks,
            [
                OspfNetwork {
                    prefix: "10.0.12.0".into(),
                    wildcard: "0.0.0.3".into(),
                    area: "0".into(),
                },
                OspfNetwork {
                    prefix: "10.1.0.0".into(),
                    wildcard: "0.0.255.255".into(),
                    area: "10".into(),
                },
            ]
        );
    }

    #[test]
    fn reads_vrf_process() {
        let config = cisco::parse(CONFIG);
        let procs = ospf_processes(&config);
        assert_eq!(procs[1].process_id, "2");
        assert_eq!(procs[1].vrf.as_deref(), Some("CUSTOMER-A"));
        assert_eq!(procs[1].passive_interfaces, ["Loopback10"]);
    }

    #[test]
    fn ospfv3_is_not_matched() {
        let config = cisco::parse(CONFIG);
        assert!(OspfProcess::from_block(&config.blocks[2]).is_none());
    }
}
