//! `vrf <name>` definitions. The `vrf` sections under `router bgp` are modelled by [`crate::bgp::BgpVrf`].

use crate::{child_arg, line_arg};
use asterust::config::{Config, ConfigBlock, ConfigLine};

/// An `address-family <afi> <safi>` section of a VRF definition.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct VrfAddressFamily {
    /// e.g. `"ipv4 unicast"`.
    pub name: String,
    pub maximum_prefix: Option<u32>,
    pub import_route_policy: Option<String>,
    pub export_route_policy: Option<String>,
    pub import_route_targets: Vec<String>,
    pub export_route_targets: Vec<String>,
}

/// An IOS XR VRF definition.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Vrf {
    pub name: String,
    pub description: Option<String>,
    pub address_families: Vec<VrfAddressFamily>,
}

impl Vrf {
    /// Build from a top-level `vrf <name>` block, or `None` for any other block.
    pub fn from_block(block: &ConfigBlock) -> Option<Self> {
        let root = block.get_root_line();
        let name = line_arg(&root.text, "vrf").filter(|n| !n.is_empty())?;
        Some(Self {
            name: name.to_string(),
            description: child_arg(root, "description").map(str::to_string),
            address_families: root
                .children
                .iter()
                .filter_map(|c| Some(parse_address_family(c, line_arg(&c.text, "address-family")?)))
                .collect(),
        })
    }
}

/// Every top-level `vrf` definition in `config`.
pub fn vrfs(config: &Config) -> Vec<Vrf> {
    config.blocks.iter().filter_map(Vrf::from_block).collect()
}

fn parse_address_family(line: &ConfigLine, name: &str) -> VrfAddressFamily {
    VrfAddressFamily {
        name: name.to_string(),
        maximum_prefix: child_arg(line, "maximum prefix").and_then(|a| a.split_whitespace().next()?.parse().ok()),
        import_route_policy: child_arg(line, "import route-policy").map(str::to_string),
        export_route_policy: child_arg(line, "export route-policy").map(str::to_string),
        import_route_targets: route_targets(line, "import"),
        export_route_targets: route_targets(line, "export"),
    }
}

/// Route targets listed as a block under `<direction> route-target` or inline after it.
fn route_targets(line: &ConfigLine, direction: &str) -> Vec<String> {
    let keyword = format!("{direction} route-target");
    let mut targets = Vec::new();
    for c in &line.children {
        if c.text == keyword {
            targets.extend(c.children.iter().map(|t| t.text.clone()));
        } else if let Some(target) = line_arg(&c.text, &keyword) {
            targets.push(target.to_string());
        }
    }
    targets
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::cisco;

    // Adapted from Cisco's "Implementing BGP on Cisco IOS XR Software" (XR 12000, R4.1), section
    // "Defining the Virtual Routing and Forwarding Tables in Provider Edge Routers"; the guide's
    // export route-target `123;234` is a typo and is written `123:234` here. See the README "Sources".
    const GUIDE_VRF: &str = "
vrf vrf_pe
 address-family ipv4 unicast
  maximum prefix 2300
  import route-policy policy_a
  import route-target
   234:222
  !
  export route-policy policy_b
  export route-target
   123:234
  !
 !
!
";

    #[test]
    fn guide_vrf_definition() {
        let vrfs = vrfs(&cisco::parse(GUIDE_VRF));
        assert_eq!(vrfs.len(), 1);

        let vrf = &vrfs[0];
        assert_eq!(vrf.name, "vrf_pe");
        let af = &vrf.address_families[0];
        assert_eq!(af.name, "ipv4 unicast");
        assert_eq!(af.maximum_prefix, Some(2300));
        assert_eq!(af.import_route_policy.as_deref(), Some("policy_a"));
        assert_eq!(af.export_route_policy.as_deref(), Some("policy_b"));
        assert_eq!(af.import_route_targets, ["234:222"]);
        assert_eq!(af.export_route_targets, ["123:234"]);
    }

    // Not from the guide: description, a second address family and inline route-targets.
    #[test]
    fn description_second_family_and_inline_route_targets() {
        let config = cisco::parse(
            "
vrf CUSTOMER-A
 description Customer A
 address-family ipv4 unicast
  import route-target 65000:1
  export route-target 65000:1
 !
 address-family ipv6 unicast
  import route-target
   65000:6
   65000:7
  !
 !
!
",
        );
        let vrf = Vrf::from_block(&config.blocks[0]).unwrap();
        assert_eq!(vrf.description.as_deref(), Some("Customer A"));
        let names: Vec<_> = vrf.address_families.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["ipv4 unicast", "ipv6 unicast"]);
        assert_eq!(vrf.address_families[0].import_route_targets, ["65000:1"]);
        assert_eq!(vrf.address_families[0].export_route_targets, ["65000:1"]);
        assert_eq!(vrf.address_families[1].import_route_targets, ["65000:6", "65000:7"]);
        assert!(vrf.address_families[1].export_route_targets.is_empty());
    }

    #[test]
    fn non_vrf_block_is_none() {
        let config = cisco::parse("router bgp 65000\n bgp router-id 1.1.1.1\n");
        assert!(Vrf::from_block(&config.blocks[0]).is_none());
    }
}
