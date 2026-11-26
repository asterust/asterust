//! [`JunosDevice`]: one parsed JunOS config with every supported view as a getter.

use crate::bgp::{bgp, Bgp};
use crate::firewall::{firewall, Firewall};
use crate::interface::{interfaces, Interface};
use crate::isis::{isis, Isis};
use crate::ospf::{ospf, Ospf};
use crate::policy_options::{policy_options, PolicyOptions};
use crate::routing_instance::{routing_instances, RoutingInstance};
use crate::routing_options::{routing_options, RoutingOptions};
use asterust::config::{juniper, Config};

/// A parsed JunOS `show configuration` (brace format).
#[derive(Clone, Debug)]
pub struct JunosDevice {
    config: Config,
}

impl JunosDevice {
    /// Parse raw brace-format `show configuration` text.
    pub fn parse(config_text: &str) -> Self {
        Self::from_config(juniper::parse_braces(config_text))
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

    pub fn ospf(&self) -> Option<Ospf> {
        ospf(&self.config)
    }

    pub fn bgp(&self) -> Option<Bgp> {
        bgp(&self.config)
    }

    pub fn isis(&self) -> Option<Isis> {
        isis(&self.config)
    }

    pub fn routing_instances(&self) -> Vec<RoutingInstance> {
        routing_instances(&self.config)
    }

    pub fn routing_options(&self) -> Option<RoutingOptions> {
        routing_options(&self.config)
    }

    pub fn policy_options(&self) -> Option<PolicyOptions> {
        policy_options(&self.config)
    }

    pub fn firewall(&self) -> Option<Firewall> {
        firewall(&self.config)
    }
}

impl asterust::device::Device for JunosDevice {
    fn parse(config_text: &str) -> Self {
        Self::parse(config_text)
    }

    fn from_config(config: Config) -> Self {
        Self::from_config(config)
    }

    fn config(&self) -> &Config {
        Self::config(self)
    }
}
