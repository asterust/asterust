//! [`IosXrDevice`]: one parsed IOS XR config with every supported block type as a getter.

use crate::bgp::{bgp_processes, BgpProcess};
use crate::interface::{interfaces, Interface};
use crate::isis::{isis_instances, IsisInstance};
use crate::ospf::{ospf_processes, OspfProcess};
use crate::vrf::{vrfs, Vrf};
use asterust::config::{cisco, Config};

/// A parsed IOS XR running config.
#[derive(Clone, Debug)]
pub struct IosXrDevice {
    config: Config,
}

impl IosXrDevice {
    /// Parse raw `show running-config` text.
    pub fn parse(config_text: &str) -> Self {
        Self::from_config(cisco::parse(config_text))
    }

    /// Wrap an already-parsed [`Config`].
    pub fn from_config(config: Config) -> Self {
        Self { config }
    }

    /// The underlying generic config tree.
    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn interfaces(&self) -> Vec<Interface> {
        interfaces(&self.config)
    }

    pub fn ospf(&self) -> Vec<OspfProcess> {
        ospf_processes(&self.config)
    }

    pub fn bgp(&self) -> Vec<BgpProcess> {
        bgp_processes(&self.config)
    }

    pub fn vrfs(&self) -> Vec<Vrf> {
        vrfs(&self.config)
    }

    pub fn isis(&self) -> Vec<IsisInstance> {
        isis_instances(&self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_every_block_type() {
        let device = IosXrDevice::parse(
            "
interface Loopback0
 ipv4 address 10.255.0.1 255.255.255.255
!
router isis CORE
 net 49.0001.0100.2550.0001.00
!
router ospf 1
 router-id 10.255.0.1
!
router bgp 65000
 neighbor 10.255.0.2
  remote-as 65000
 !
!
vrf CUSTOMER-A
 address-family ipv4 unicast
 !
!
",
        );
        assert_eq!(device.interfaces()[0].name, "Loopback0");
        assert_eq!(device.isis()[0].tag, "CORE");
        assert_eq!(device.vrfs()[0].name, "CUSTOMER-A");
        assert_eq!(device.ospf()[0].process_id, "1");
        assert_eq!(device.bgp()[0].neighbors[0].address, "10.255.0.2");
        assert_eq!(device.config().get_block_count(), 5);
    }
}
