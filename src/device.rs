//! Traits that platform crates (`asterust-iosxr`, `asterust-iosxe`, ...) implement, so code can be
//! written once over any platform's device.
//!
//! A platform implements [`BlockView`] for each typed block it extracts and [`Device`] for the
//! wrapper around a parsed [`Config`]. The `Has*` traits then supply the getters: a platform only
//! names its block types and gets `interfaces()`, `ospf()`, ... for free. A platform that lacks a
//! feature (say, no IS-IS) simply doesn't implement that trait.
//!
//! Field types differ per platform (IOS XE interfaces carry per-interface OSPF settings, IOS XR
//! interfaces don't), so the traits share behaviour, not data.

use crate::config::{Config, ConfigBlock};

/// A typed view of one top-level config block.
pub trait BlockView: Sized {
    /// Build from a top-level block, or `None` if the block isn't this kind.
    fn from_block(block: &ConfigBlock) -> Option<Self>;

    /// Every block in `config` that is this kind, in config order.
    fn find_all(config: &Config) -> Vec<Self> {
        config.blocks.iter().filter_map(Self::from_block).collect()
    }
}

/// A parsed config for one platform.
pub trait Device: Sized {
    /// Parse raw `show running-config` text using the platform's syntax.
    fn parse(config_text: &str) -> Self;

    /// Wrap an already-parsed [`Config`].
    fn from_config(config: Config) -> Self;

    /// The underlying generic config tree.
    fn config(&self) -> &Config;
}

/// A device whose config has `interface` blocks.
pub trait HasInterfaces: Device {
    type Interface: BlockView;

    fn interfaces(&self) -> Vec<Self::Interface> {
        Self::Interface::find_all(self.config())
    }
}

/// A device whose config has OSPF processes.
pub trait HasOspf: Device {
    type Ospf: BlockView;

    fn ospf(&self) -> Vec<Self::Ospf> {
        Self::Ospf::find_all(self.config())
    }
}

/// A device whose config has BGP processes.
pub trait HasBgp: Device {
    type Bgp: BlockView;

    fn bgp(&self) -> Vec<Self::Bgp> {
        Self::Bgp::find_all(self.config())
    }
}

/// A device whose config has IS-IS instances.
pub trait HasIsis: Device {
    type Isis: BlockView;

    fn isis(&self) -> Vec<Self::Isis> {
        Self::Isis::find_all(self.config())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::cisco;

    struct Vlan(String);

    impl BlockView for Vlan {
        fn from_block(block: &ConfigBlock) -> Option<Self> {
            block.get_root_line().arg("vlan").map(|id| Vlan(id.to_string()))
        }
    }

    struct ToyDevice(Config);

    impl Device for ToyDevice {
        fn parse(config_text: &str) -> Self {
            Self::from_config(cisco::parse(config_text))
        }

        fn from_config(config: Config) -> Self {
            Self(config)
        }

        fn config(&self) -> &Config {
            &self.0
        }
    }

    impl HasInterfaces for ToyDevice {
        type Interface = Vlan;
    }

    fn vlan_ids<D: HasInterfaces<Interface = Vlan>>(device: &D) -> Vec<String> {
        device.interfaces().into_iter().map(|v| v.0).collect()
    }

    #[test]
    fn find_all_skips_other_blocks_in_order() {
        let config = cisco::parse("vlan 10\n name a\n!\nhostname r1\n!\nvlan 20\n name b\n!\n");
        let ids: Vec<_> = Vlan::find_all(&config).into_iter().map(|v| v.0).collect();
        assert_eq!(ids, ["10", "20"]);
    }

    #[test]
    fn has_traits_supply_getters_for_generic_code() {
        let device = ToyDevice::parse("vlan 10\n!\nvlan 20\n!\n");
        assert_eq!(vlan_ids(&device), ["10", "20"]);
        assert_eq!(device.config().get_block_count(), 2);
    }
}
