//! `interface <name>` blocks.

use crate::{child_arg, child_args, line_arg};
use asterust::config::{Config, ConfigBlock};

/// An IOS XR interface. Address fields hold the raw argument text
/// (e.g. `"10.0.0.1 255.255.255.0"`, `"2001:db8::1/64"`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interface {
    pub name: String,
    pub description: Option<String>,
    pub vrf: Option<String>,
    pub ipv4_addresses: Vec<String>,
    pub ipv6_addresses: Vec<String>,
    pub shutdown: bool,
}

impl Interface {
    /// Build from a top-level `interface <name>` block, or `None` for any other block.
    pub fn from_block(block: &ConfigBlock) -> Option<Self> {
        let root = block.get_root_line();
        let name = line_arg(&root.text, "interface").filter(|n| !n.is_empty())?;
        Some(Self {
            name: name.to_string(),
            description: child_arg(root, "description").map(str::to_string),
            vrf: child_arg(root, "vrf").map(str::to_string),
            ipv4_addresses: child_args(root, "ipv4 address")
                .into_iter()
                .map(str::to_string)
                .collect(),
            ipv6_addresses: child_args(root, "ipv6 address")
                .into_iter()
                .map(str::to_string)
                .collect(),
            shutdown: root.children.iter().any(|c| c.text == "shutdown"),
        })
    }
}

/// Every `interface` block in `config`.
pub fn interfaces(config: &Config) -> Vec<Interface> {
    config.blocks.iter().filter_map(Interface::from_block).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::cisco;

    const CONFIG: &str = "
interface Loopback0
 description Router ID
 ipv4 address 10.255.0.1 255.255.255.255
!
interface TenGigE0/0/0/1
 description to core1 TenGigE0/0/0/3
 vrf CUSTOMER-A
 ipv4 address 10.0.12.1 255.255.255.252
 ipv4 address 10.0.13.1 255.255.255.252 secondary
 ipv6 address 2001:db8:12::1/64
!
interface TenGigE0/0/0/2
 shutdown
!
router ospf 1
 router-id 10.255.0.1
!
";

    #[test]
    fn extracts_full_interface() {
        let config = cisco::parse(CONFIG);
        let ifaces = interfaces(&config);
        assert_eq!(ifaces.len(), 3);

        let ten_gig = &ifaces[1];
        assert_eq!(ten_gig.name, "TenGigE0/0/0/1");
        assert_eq!(ten_gig.description.as_deref(), Some("to core1 TenGigE0/0/0/3"));
        assert_eq!(ten_gig.vrf.as_deref(), Some("CUSTOMER-A"));
        assert_eq!(
            ten_gig.ipv4_addresses,
            ["10.0.12.1 255.255.255.252", "10.0.13.1 255.255.255.252 secondary"]
        );
        assert_eq!(ten_gig.ipv6_addresses, ["2001:db8:12::1/64"]);
        assert!(!ten_gig.shutdown);
    }

    #[test]
    fn detects_shutdown_and_missing_fields() {
        let config = cisco::parse(CONFIG);
        let ifaces = interfaces(&config);
        assert!(ifaces[2].shutdown);
        assert_eq!(ifaces[2].description, None);
        assert!(ifaces[2].ipv4_addresses.is_empty());
    }

    #[test]
    fn non_interface_block_is_none() {
        let config = cisco::parse(CONFIG);
        assert!(Interface::from_block(&config.blocks[3]).is_none());
    }
}
