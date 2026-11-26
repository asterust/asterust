//! `router isis [tag]` blocks.

use crate::{child_args, line_arg};
use asterust::config::{Config, ConfigBlock};

/// An IOS XE IS-IS process. Interfaces enabled with `ip router isis` are on
/// [`crate::interface::Interface`], not here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsisInstance {
    /// `None` for the untagged process (plain `router isis`).
    pub tag: Option<String>,
    pub nets: Vec<String>,
}

impl IsisInstance {
    /// Build from a top-level `router isis` block, or `None` for any other block.
    pub fn from_block(block: &ConfigBlock) -> Option<Self> {
        let root = block.get_root_line();
        let tag = if root.text == "router isis" {
            None
        } else {
            Some(
                line_arg(&root.text, "router isis")
                    .filter(|t| !t.is_empty())?
                    .to_string(),
            )
        };
        Some(Self {
            tag,
            nets: child_args(root, "net").into_iter().map(str::to_string).collect(),
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
 net 49.0001.0100.2550.0001.00
 is-type level-2-only
 metric-style wide
!
router isis
 net 49.0002.0100.2550.0001.00
!
router isis-extra
!
";

    #[test]
    fn extracts_tagged_and_untagged_instances() {
        let config = cisco::parse(CONFIG);
        let instances = isis_instances(&config);
        assert_eq!(instances.len(), 2);

        assert_eq!(instances[0].tag.as_deref(), Some("CORE"));
        assert_eq!(instances[0].nets, ["49.0001.0100.2550.0001.00"]);
        assert_eq!(instances[1].tag, None);
        assert_eq!(instances[1].nets, ["49.0002.0100.2550.0001.00"]);
    }

    #[test]
    fn non_isis_block_is_none() {
        let config = cisco::parse("interface Loopback0\n description x\n");
        assert!(IsisInstance::from_block(&config.blocks[0]).is_none());
    }
}
