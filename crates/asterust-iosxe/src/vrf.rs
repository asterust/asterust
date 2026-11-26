//! `vrf definition <name>` and legacy `ip vrf <name>` blocks.

use crate::{child_arg, child_args, line_arg};
use asterust::config::{Config, ConfigBlock, ConfigLine};

/// Route targets and maps for one address family of a VRF.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct VrfAddressFamily {
    /// e.g. `"ipv4"` or `"ipv6"`.
    pub name: String,
    /// `route-target both` entries appear in both lists.
    pub import_route_targets: Vec<String>,
    pub export_route_targets: Vec<String>,
    pub import_map: Option<String>,
    pub export_map: Option<String>,
}

/// An IOS XE VRF. The legacy `ip vrf` form has no address-family sections; its route targets and
/// maps are reported as a single `"ipv4"` address family.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Vrf {
    pub name: String,
    pub description: Option<String>,
    pub rd: Option<String>,
    pub address_families: Vec<VrfAddressFamily>,
    /// `true` for `ip vrf <name>`, `false` for `vrf definition <name>`.
    pub legacy: bool,
}

impl Vrf {
    /// Build from a top-level `vrf definition <name>` or `ip vrf <name>` block, or `None` for any other block.
    pub fn from_block(block: &ConfigBlock) -> Option<Self> {
        let root = block.get_root_line();
        let (name, legacy) = match line_arg(&root.text, "vrf definition") {
            Some(name) => (name, false),
            None => (line_arg(&root.text, "ip vrf")?, true),
        };
        if name.is_empty() {
            return None;
        }

        let mut address_families: Vec<_> = root
            .children
            .iter()
            .filter_map(|c| Some(parse_address_family(c, line_arg(&c.text, "address-family")?)))
            .collect();
        if legacy {
            address_families.push(parse_address_family(root, "ipv4"));
        }

        Some(Self {
            name: name.to_string(),
            description: child_arg(root, "description").map(str::to_string),
            rd: child_arg(root, "rd").map(str::to_string),
            address_families,
            legacy,
        })
    }
}

/// Every `vrf definition` and legacy `ip vrf` block in `config`.
pub fn vrfs(config: &Config) -> Vec<Vrf> {
    config.blocks.iter().filter_map(Vrf::from_block).collect()
}

fn parse_address_family(line: &ConfigLine, name: &str) -> VrfAddressFamily {
    let mut af = VrfAddressFamily {
        name: name.to_string(),
        import_route_targets: Vec::new(),
        export_route_targets: Vec::new(),
        import_map: child_arg(line, "import map").map(str::to_string),
        export_map: child_arg(line, "export map").map(str::to_string),
    };
    for arg in child_args(line, "route-target") {
        let mut words = arg.split_whitespace();
        let (Some(direction), Some(target)) = (words.next(), words.next()) else {
            continue;
        };
        if matches!(direction, "import" | "both") {
            af.import_route_targets.push(target.to_string());
        }
        if matches!(direction, "export" | "both") {
            af.export_route_targets.push(target.to_string());
        }
    }
    af
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::cisco;

    // Written from the IOS XE `vrf definition` / `ip vrf` command syntax. Not checked against a Cisco
    // guide or a real device yet; see the README "Sources".
    const CONFIG: &str = "
vrf definition CUSTOMER-A
 description Customer A
 rd 65000:1
 !
 address-family ipv4
  route-target export 65000:1
  route-target import 65000:1
  route-target both 65000:100
  import map FROM-CORE
  export map TO-CORE
 exit-address-family
 !
 address-family ipv6
  route-target both 65000:6
 exit-address-family
!
ip vrf LEGACY
 rd 65000:9
 route-target export 65000:9
 route-target import 65000:9
 import map LEGACY-IN
!
interface Loopback0
 ip vrf forwarding LEGACY
!
";

    #[test]
    fn vrf_definition_with_address_families() {
        let vrfs = vrfs(&cisco::parse(CONFIG));
        assert_eq!(vrfs.len(), 2);

        let vrf = &vrfs[0];
        assert_eq!(vrf.name, "CUSTOMER-A");
        assert_eq!(vrf.description.as_deref(), Some("Customer A"));
        assert_eq!(vrf.rd.as_deref(), Some("65000:1"));
        assert!(!vrf.legacy);

        let names: Vec<_> = vrf.address_families.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["ipv4", "ipv6"]);
        let ipv4 = &vrf.address_families[0];
        assert_eq!(ipv4.import_route_targets, ["65000:1", "65000:100"]);
        assert_eq!(ipv4.export_route_targets, ["65000:1", "65000:100"]);
        assert_eq!(ipv4.import_map.as_deref(), Some("FROM-CORE"));
        assert_eq!(ipv4.export_map.as_deref(), Some("TO-CORE"));

        let ipv6 = &vrf.address_families[1];
        assert_eq!(ipv6.import_route_targets, ["65000:6"]);
        assert_eq!(ipv6.export_route_targets, ["65000:6"]);
    }

    #[test]
    fn legacy_ip_vrf_is_reported_as_one_ipv4_family() {
        let vrfs = vrfs(&cisco::parse(CONFIG));
        let legacy = &vrfs[1];
        assert_eq!(legacy.name, "LEGACY");
        assert!(legacy.legacy);
        assert_eq!(legacy.rd.as_deref(), Some("65000:9"));
        assert_eq!(legacy.address_families.len(), 1);
        let af = &legacy.address_families[0];
        assert_eq!(af.name, "ipv4");
        assert_eq!(af.import_route_targets, ["65000:9"]);
        assert_eq!(af.export_route_targets, ["65000:9"]);
        assert_eq!(af.import_map.as_deref(), Some("LEGACY-IN"));
    }

    #[test]
    fn interface_vrf_forwarding_is_not_a_vrf_block() {
        let config = cisco::parse(CONFIG);
        assert!(Vrf::from_block(&config.blocks[2]).is_none());
    }
}
