//! `interface <name>` blocks.

use crate::{child_arg, child_args, line_arg};
use asterust::config::{Config, ConfigBlock, ConfigLine};

/// An IOS XE interface. Address fields hold the raw argument text
/// (e.g. `"10.0.0.1 255.255.255.0"`, `"2001:db8::1/64"`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interface {
    pub name: String,
    pub description: Option<String>,
    pub vrf: Option<String>,
    pub ipv4_addresses: Vec<String>,
    pub ipv6_addresses: Vec<String>,
    pub shutdown: bool,
    /// Process id and area from `ip ospf <process> area <area>`.
    pub ospf_process_id: Option<String>,
    pub ospf_area: Option<String>,
    /// Tag from `ip router isis [tag]`; empty string for the untagged process.
    pub isis_tag: Option<String>,
}

impl Interface {
    /// Build from a top-level `interface <name>` block, or `None` for any other block.
    pub fn from_block(block: &ConfigBlock) -> Option<Self> {
        let root = block.get_root_line();
        let name = line_arg(&root.text, "interface").filter(|n| !n.is_empty())?;
        let (ospf_process_id, ospf_area) = ospf_enablement(root).unzip();
        Some(Self {
            name: name.to_string(),
            description: child_arg(root, "description").map(str::to_string),
            vrf: child_arg(root, "vrf forwarding")
                .or_else(|| child_arg(root, "ip vrf forwarding"))
                .map(str::to_string),
            ipv4_addresses: child_args(root, "ip address").into_iter().map(str::to_string).collect(),
            ipv6_addresses: child_args(root, "ipv6 address")
                .into_iter()
                .map(str::to_string)
                .collect(),
            shutdown: root.children.iter().any(|c| c.text == "shutdown"),
            ospf_process_id,
            ospf_area,
            isis_tag: isis_tag(root),
        })
    }
}

/// `(process id, area)` from an `ip ospf <process> area <area>` line.
fn ospf_enablement(root: &ConfigLine) -> Option<(String, String)> {
    child_args(root, "ip ospf").into_iter().find_map(|arg| {
        let mut words = arg.split_whitespace();
        let process_id = words.next()?;
        if words.next()? != "area" {
            return None;
        }
        let area = words.next()?;
        Some((process_id.to_string(), area.to_string()))
    })
}

fn isis_tag(root: &ConfigLine) -> Option<String> {
    root.children.iter().find_map(|c| {
        if c.text == "ip router isis" {
            Some(String::new())
        } else {
            line_arg(&c.text, "ip router isis").map(str::to_string)
        }
    })
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
 ip address 10.255.0.1 255.255.255.255
 ip ospf 1 area 0
 ip router isis CORE
!
interface GigabitEthernet0/0/0
 description to core1 Gi0/0/3
 vrf forwarding CUSTOMER-A
 ip address 10.0.12.1 255.255.255.252
 ip address 10.0.13.1 255.255.255.252 secondary
 ipv6 address 2001:db8:12::1/64
 ip ospf network point-to-point
 ip ospf cost 10
 negotiation auto
!
interface GigabitEthernet0/0/1
 no ip address
 shutdown
!
interface GigabitEthernet0/0/2
 ip vrf forwarding LEGACY
 ip address 192.0.2.1 255.255.255.0
 ip router isis
!
router ospf 1
 router-id 10.255.0.1
!
";

    #[test]
    fn extracts_full_interface() {
        let config = cisco::parse(CONFIG);
        let ifaces = interfaces(&config);
        assert_eq!(ifaces.len(), 4);

        let gig = &ifaces[1];
        assert_eq!(gig.name, "GigabitEthernet0/0/0");
        assert_eq!(gig.description.as_deref(), Some("to core1 Gi0/0/3"));
        assert_eq!(gig.vrf.as_deref(), Some("CUSTOMER-A"));
        assert_eq!(
            gig.ipv4_addresses,
            ["10.0.12.1 255.255.255.252", "10.0.13.1 255.255.255.252 secondary"]
        );
        assert_eq!(gig.ipv6_addresses, ["2001:db8:12::1/64"]);
        assert!(!gig.shutdown);
    }

    #[test]
    fn reads_per_interface_ospf_and_isis() {
        let config = cisco::parse(CONFIG);
        let ifaces = interfaces(&config);

        assert_eq!(ifaces[0].ospf_process_id.as_deref(), Some("1"));
        assert_eq!(ifaces[0].ospf_area.as_deref(), Some("0"));
        assert_eq!(ifaces[0].isis_tag.as_deref(), Some("CORE"));

        // `ip ospf cost` / `ip ospf network` are not area enablement.
        assert_eq!(ifaces[1].ospf_area, None);
        assert_eq!(ifaces[1].isis_tag, None);

        assert_eq!(ifaces[3].isis_tag.as_deref(), Some(""));
    }

    #[test]
    fn accepts_legacy_vrf_syntax_and_detects_shutdown() {
        let config = cisco::parse(CONFIG);
        let ifaces = interfaces(&config);
        assert_eq!(ifaces[3].vrf.as_deref(), Some("LEGACY"));
        assert!(ifaces[2].shutdown);
        assert!(ifaces[2].ipv4_addresses.is_empty());
    }

    #[test]
    fn non_interface_block_is_none() {
        let config = cisco::parse(CONFIG);
        assert!(Interface::from_block(&config.blocks[4]).is_none());
    }
}
