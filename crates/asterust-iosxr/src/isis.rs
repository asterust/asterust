//! `router isis <tag>` blocks.

use crate::{child_args, line_arg};
use asterust::config::{Config, ConfigBlock};

/// An IOS XR IS-IS instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsisInstance {
    pub tag: String,
    pub nets: Vec<String>,
    pub interfaces: Vec<String>,
}

impl IsisInstance {
    /// Build from a top-level `router isis <tag>` block, or `None` for any other block.
    pub fn from_block(block: &ConfigBlock) -> Option<Self> {
        let root = block.get_root_line();
        let tag = line_arg(&root.text, "router isis").filter(|t| !t.is_empty())?;
        Some(Self {
            tag: tag.to_string(),
            nets: child_args(root, "net").into_iter().map(str::to_string).collect(),
            interfaces: child_args(root, "interface").into_iter().map(str::to_string).collect(),
        })
    }
}

/// Every `router isis` block in `config`.
pub fn isis_instances(config: &Config) -> Vec<IsisInstance> {
    config.blocks.iter().filter_map(IsisInstance::from_block).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::cisco;

    const CONFIG: &str = "
router isis CORE
 is-type level-2-only
 net 49.0001.0100.2550.0001.00
 address-family ipv4 unicast
  metric-style wide
 !
 interface Loopback0
  passive
  address-family ipv4 unicast
  !
 !
 interface TenGigE0/0/0/1
  point-to-point
 !
!
";

    #[test]
    fn extracts_tag_net_and_interfaces() {
        let config = cisco::parse(CONFIG);
        let instances = isis_instances(&config);
        assert_eq!(instances.len(), 1);

        let isis = &instances[0];
        assert_eq!(isis.tag, "CORE");
        assert_eq!(isis.nets, ["49.0001.0100.2550.0001.00"]);
        assert_eq!(isis.interfaces, ["Loopback0", "TenGigE0/0/0/1"]);
    }

    #[test]
    fn non_isis_block_is_none() {
        let config = cisco::parse("interface Loopback0\n description x\n");
        assert!(IsisInstance::from_block(&config.blocks[0]).is_none());
    }
}
