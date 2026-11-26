//! `router ospf <process>` blocks.

use crate::{child_arg, line_arg};
use asterust::config::{Config, ConfigBlock};

/// An OSPF area and the interfaces configured under it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OspfArea {
    pub id: String,
    pub interfaces: Vec<String>,
}

/// An IOS XR OSPF process.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OspfProcess {
    pub process_id: String,
    pub router_id: Option<String>,
    pub areas: Vec<OspfArea>,
}

impl OspfProcess {
    /// Build from a top-level `router ospf <process>` block, or `None` for any other block.
    pub fn from_block(block: &ConfigBlock) -> Option<Self> {
        let root = block.get_root_line();
        let process_id = line_arg(&root.text, "router ospf").filter(|p| !p.is_empty())?;
        let areas = root
            .children
            .iter()
            .filter_map(|c| {
                let id = line_arg(&c.text, "area")?;
                Some(OspfArea {
                    id: id.to_string(),
                    interfaces: crate::child_args(c, "interface")
                        .into_iter()
                        .map(str::to_string)
                        .collect(),
                })
            })
            .collect();
        Some(Self {
            process_id: process_id.to_string(),
            router_id: child_arg(root, "router-id").map(str::to_string),
            areas,
        })
    }
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
 area 0
  interface Loopback0
   passive enable
  !
  interface TenGigE0/0/0/1
   cost 10
   network point-to-point
  !
 !
 area 0.0.0.10
  interface TenGigE0/0/0/2
  !
 !
!
router ospfv3 2
 router-id 10.255.0.1
!
";

    #[test]
    fn extracts_process_areas_and_interfaces() {
        let config = cisco::parse(CONFIG);
        let procs = ospf_processes(&config);
        assert_eq!(procs.len(), 1);

        let ospf = &procs[0];
        assert_eq!(ospf.process_id, "1");
        assert_eq!(ospf.router_id.as_deref(), Some("10.255.0.1"));
        assert_eq!(ospf.areas.len(), 2);
        assert_eq!(ospf.areas[0].id, "0");
        assert_eq!(ospf.areas[0].interfaces, ["Loopback0", "TenGigE0/0/0/1"]);
        assert_eq!(ospf.areas[1].id, "0.0.0.10");
        assert_eq!(ospf.areas[1].interfaces, ["TenGigE0/0/0/2"]);
    }

    #[test]
    fn ospfv3_is_not_matched() {
        let config = cisco::parse(CONFIG);
        assert!(OspfProcess::from_block(&config.blocks[1]).is_none());
    }
}
