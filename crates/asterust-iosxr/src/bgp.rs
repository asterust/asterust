//! `router bgp <asn>` blocks.
//!
//! The model follows the IOS XR BGP configuration structure: neighbors, neighbor groups, session
//! groups and address-family groups (templates a neighbor inherits through `use`), and VRFs. See the
//! README "Sources" section for the Cisco documentation the structure and test fixtures come from.

use crate::{child_arg, child_args, line_arg};
use asterust::config::{Config, ConfigBlock, ConfigLine};
use std::ops::Deref;

/// Keepalive and hold-time from `timers [bgp] <keepalive> <hold-time>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpTimers {
    pub keepalive: u32,
    pub hold_time: u32,
}

/// Address-family-dependent settings. The same commands are valid under a neighbor, a neighbor
/// group and an address-family group.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpAfSettings {
    pub route_policy_in: Option<String>,
    pub route_policy_out: Option<String>,
    pub route_reflector_client: bool,
    pub next_hop_self: bool,
    pub soft_reconfiguration_inbound: bool,
    pub send_community_ebgp: bool,
    pub send_extended_community_ebgp: bool,
    pub default_originate: bool,
    pub remove_private_as: bool,
    pub as_override: bool,
    pub allowas_in: Option<u32>,
    pub site_of_origin: Option<String>,
    pub maximum_prefix: Option<u32>,
    pub weight: Option<u32>,
    pub use_af_group: Option<String>,
}

/// An `address-family <afi> <safi>` section under a neighbor or neighbor group.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpNeighborAddressFamily {
    /// e.g. `"ipv4 unicast"`.
    pub name: String,
    pub settings: BgpAfSettings,
}

/// Address-family-independent settings. The same commands are valid under a neighbor, a neighbor
/// group and a session group. Passwords and keys are deliberately not extracted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpPeerSettings {
    pub remote_as: Option<String>,
    pub description: Option<String>,
    pub update_source: Option<String>,
    pub ebgp_multihop: Option<u32>,
    pub advertisement_interval: Option<u32>,
    pub timers: Option<BgpTimers>,
    pub shutdown: bool,
    pub use_neighbor_group: Option<String>,
    pub use_session_group: Option<String>,
    pub address_families: Vec<BgpNeighborAddressFamily>,
}

/// A `neighbor <address>` block. Peer settings are reachable directly (`neighbor.remote_as`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpNeighbor {
    pub address: String,
    pub settings: BgpPeerSettings,
}

/// A `neighbor-group <name>` template.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpNeighborGroup {
    pub name: String,
    pub settings: BgpPeerSettings,
}

/// A `session-group <name>` template (address-family-independent settings only).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpSessionGroup {
    pub name: String,
    pub settings: BgpPeerSettings,
}

/// An `af-group <name> address-family <afi> <safi>` template.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpAfGroup {
    pub name: String,
    /// e.g. `"ipv4 unicast"`.
    pub address_family: String,
    pub settings: BgpAfSettings,
}

macro_rules! deref_to_peer_settings {
    ($($ty:ty),*) => {$(
        impl Deref for $ty {
            type Target = BgpPeerSettings;

            fn deref(&self) -> &BgpPeerSettings {
                &self.settings
            }
        }
    )*};
}

deref_to_peer_settings!(BgpNeighbor, BgpNeighborGroup, BgpSessionGroup);

/// A router- or VRF-level `address-family <afi> <safi>` section. Values are the raw argument text
/// (e.g. `"10.2.0.0/24 route-policy foo"`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpAddressFamily {
    /// e.g. `"ipv4 unicast"`.
    pub name: String,
    pub networks: Vec<String>,
    pub aggregate_addresses: Vec<String>,
    pub redistributions: Vec<String>,
}

/// A `vrf <name>` section under `router bgp`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpVrf {
    pub name: String,
    pub rd: Option<String>,
    pub router_id: Option<String>,
    pub address_families: Vec<BgpAddressFamily>,
    pub neighbors: Vec<BgpNeighbor>,
}

/// An IOS XR BGP process. `asn` is kept as text since it may be asdot (`"1.10"`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpProcess {
    pub asn: String,
    pub router_id: Option<String>,
    /// `timers bgp <keepalive> <hold-time>` (default for all neighbors).
    pub timers: Option<BgpTimers>,
    pub default_local_preference: Option<u32>,
    pub cluster_id: Option<String>,
    pub confederation_identifier: Option<String>,
    pub confederation_peers: Vec<String>,
    pub nsr: bool,
    pub address_families: Vec<BgpAddressFamily>,
    pub neighbors: Vec<BgpNeighbor>,
    pub neighbor_groups: Vec<BgpNeighborGroup>,
    pub session_groups: Vec<BgpSessionGroup>,
    pub af_groups: Vec<BgpAfGroup>,
    pub vrfs: Vec<BgpVrf>,
}

impl BgpProcess {
    /// Build from a top-level `router bgp <asn>` block, or `None` for any other block.
    pub fn from_block(block: &ConfigBlock) -> Option<Self> {
        let root = block.get_root_line();
        let asn = line_arg(&root.text, "router bgp")?.split_whitespace().next()?;

        let mut confederation_peers = Vec::new();
        for c in &root.children {
            if c.text == "bgp confederation peers" {
                confederation_peers.extend(c.children.iter().map(|p| p.text.clone()));
            } else if let Some(peer) = line_arg(&c.text, "bgp confederation peers") {
                confederation_peers.push(peer.to_string());
            }
        }

        Some(Self {
            asn: asn.to_string(),
            router_id: child_arg(root, "bgp router-id").map(str::to_string),
            timers: child_arg(root, "timers bgp").and_then(parse_timers),
            default_local_preference: child_arg(root, "bgp default local-preference").and_then(first_u32),
            cluster_id: child_arg(root, "bgp cluster-id").map(str::to_string),
            confederation_identifier: child_arg(root, "bgp confederation identifier").map(str::to_string),
            confederation_peers,
            nsr: root.children.iter().any(|c| c.text == "nsr"),
            address_families: collect_address_families(root),
            neighbors: collect_neighbors(root),
            neighbor_groups: named_children(root, "neighbor-group")
                .map(|(name, c)| BgpNeighborGroup {
                    name,
                    settings: parse_peer_settings(c),
                })
                .collect(),
            session_groups: named_children(root, "session-group")
                .map(|(name, c)| BgpSessionGroup {
                    name,
                    settings: parse_peer_settings(c),
                })
                .collect(),
            af_groups: root.children.iter().filter_map(parse_af_group).collect(),
            vrfs: named_children(root, "vrf")
                .map(|(name, c)| BgpVrf {
                    name,
                    rd: child_arg(c, "rd").map(str::to_string),
                    router_id: child_arg(c, "bgp router-id").map(str::to_string),
                    address_families: collect_address_families(c),
                    neighbors: collect_neighbors(c),
                })
                .collect(),
        })
    }
}

/// Every `router bgp` block in `config`.
pub fn bgp_processes(config: &Config) -> Vec<BgpProcess> {
    config.blocks.iter().filter_map(BgpProcess::from_block).collect()
}

/// Direct children of `parent` that are `<keyword> <name>`, with the name.
fn named_children<'a>(parent: &'a ConfigLine, keyword: &'a str) -> impl Iterator<Item = (String, &'a ConfigLine)> {
    parent
        .children
        .iter()
        .filter_map(move |c| Some((line_arg(&c.text, keyword)?.to_string(), c)))
}

fn collect_neighbors(parent: &ConfigLine) -> Vec<BgpNeighbor> {
    named_children(parent, "neighbor")
        .map(|(address, c)| BgpNeighbor {
            address,
            settings: parse_peer_settings(c),
        })
        .collect()
}

fn collect_address_families(parent: &ConfigLine) -> Vec<BgpAddressFamily> {
    named_children(parent, "address-family")
        .map(|(name, c)| BgpAddressFamily {
            name,
            networks: owned(child_args(c, "network")),
            aggregate_addresses: owned(child_args(c, "aggregate-address")),
            redistributions: owned(child_args(c, "redistribute")),
        })
        .collect()
}

fn parse_af_group(line: &ConfigLine) -> Option<BgpAfGroup> {
    let (name, address_family) = line_arg(&line.text, "af-group")?.split_once(" address-family ")?;
    Some(BgpAfGroup {
        name: name.trim().to_string(),
        address_family: address_family.trim().to_string(),
        settings: parse_af_settings(line),
    })
}

fn parse_peer_settings(line: &ConfigLine) -> BgpPeerSettings {
    let mut settings = BgpPeerSettings {
        remote_as: child_arg(line, "remote-as").map(str::to_string),
        description: child_arg(line, "description").map(str::to_string),
        update_source: child_arg(line, "update-source").map(str::to_string),
        ebgp_multihop: child_arg(line, "ebgp-multihop").and_then(first_u32),
        advertisement_interval: child_arg(line, "advertisement-interval").and_then(first_u32),
        timers: child_arg(line, "timers").and_then(parse_timers),
        shutdown: line.children.iter().any(|c| c.text == "shutdown"),
        use_neighbor_group: None,
        use_session_group: None,
        address_families: named_children(line, "address-family")
            .map(|(name, c)| BgpNeighborAddressFamily {
                name,
                settings: parse_af_settings(c),
            })
            .collect(),
    };
    for arg in child_args(line, "use") {
        if let Some(group) = arg.strip_prefix("neighbor-group ") {
            settings.use_neighbor_group = Some(group.trim().to_string());
        } else if let Some(group) = arg.strip_prefix("session-group ") {
            settings.use_session_group = Some(group.trim().to_string());
        }
    }
    settings
}

fn parse_af_settings(line: &ConfigLine) -> BgpAfSettings {
    let mut settings = BgpAfSettings {
        route_reflector_client: has_flag(line, "route-reflector-client"),
        next_hop_self: has_flag(line, "next-hop-self"),
        send_community_ebgp: has_flag(line, "send-community-ebgp"),
        send_extended_community_ebgp: has_flag(line, "send-extended-community-ebgp"),
        default_originate: has_flag(line, "default-originate"),
        remove_private_as: has_flag(line, "remove-private-as"),
        as_override: has_flag(line, "as-override"),
        soft_reconfiguration_inbound: child_arg(line, "soft-reconfiguration").is_some_and(|a| a.starts_with("inbound")),
        allowas_in: child_arg(line, "allowas-in").and_then(first_u32),
        site_of_origin: child_arg(line, "site-of-origin").map(str::to_string),
        maximum_prefix: child_arg(line, "maximum-prefix").and_then(first_u32),
        weight: child_arg(line, "weight").and_then(first_u32),
        ..BgpAfSettings::default()
    };
    for arg in child_args(line, "route-policy") {
        let Some((name, direction)) = arg.rsplit_once(' ') else {
            continue;
        };
        match direction {
            "in" => settings.route_policy_in = Some(name.trim().to_string()),
            "out" => settings.route_policy_out = Some(name.trim().to_string()),
            _ => {}
        }
    }
    for arg in child_args(line, "use") {
        if let Some(group) = arg.strip_prefix("af-group ") {
            settings.use_af_group = Some(group.trim().to_string());
        }
    }
    settings
}

/// Whether `line` has a child that is exactly `keyword` or `keyword` followed by arguments.
fn has_flag(line: &ConfigLine, keyword: &str) -> bool {
    line.children
        .iter()
        .any(|c| c.text == keyword || line_arg(&c.text, keyword).is_some())
}

fn first_u32(arg: &str) -> Option<u32> {
    arg.split_whitespace().next()?.parse().ok()
}

fn parse_timers(arg: &str) -> Option<BgpTimers> {
    let mut words = arg.split_whitespace();
    Some(BgpTimers {
        keepalive: words.next()?.parse().ok()?,
        hold_time: words.next()?.parse().ok()?,
    })
}

fn owned(args: Vec<&str>) -> Vec<String> {
    args.into_iter().map(str::to_string).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::cisco;

    // Fixtures are adapted from Cisco's "Implementing BGP on Cisco IOS XR Software" (XR 12000, R4.1);
    // see the crate README "Sources" section.

    /// Doc section "Enabling BGP: Example" (shortened).
    const ENABLING_BGP: &str = "
router bgp 65000
 bgp fast-external-fallover disable
 bgp confederation peers
  65001
  65002
 !
 bgp confederation identifier 1
 bgp router-id 1.1.1.1
 address-family ipv4 unicast
  aggregate-address 10.2.0.0/24 route-policy set_next_hop_agg_v4
  aggregate-address 10.3.0.0/24
  redistribute static route-policy set_next_hop_static_v4
 !
 address-family ipv4 multicast
  aggregate-address 10.2.0.0/24 route-policy set_next_hop_agg_v4
 !
 neighbor 10.0.101.60
  remote-as 65000
  address-family ipv4 unicast
  !
  address-family ipv4 multicast
  !
 !
 neighbor 10.0.101.62
  remote-as 3
  address-family ipv4 unicast
   route-policy pass-all in
   route-policy pass-all out
  !
 !
 neighbor 10.0.101.64
  remote-as 5
  update-source Loopback0
  address-family ipv4 unicast
   route-policy pass-all in
   route-policy pass-all out
  !
 !
!
";

    /// Doc section "Viewing Inherited Configurations" (sample configuration for `show bgp neighbors`).
    const TEMPLATES: &str = "
router bgp 142
 af-group GROUP_3 address-family ipv4 unicast
  next-hop-self
  route-policy POLICY_1 in
 !
 session-group GROUP_2
  advertisement-interval 15
 !
 neighbor-group GROUP_1
  use session-group GROUP_2
  ebgp-multihop 3
  address-family ipv4 unicast
   weight 100
   send-community-ebgp
  !
  address-family ipv4 multicast
   default-originate
  !
 !
 neighbor 192.168.0.1
  remote-as 2
  use neighbor-group GROUP_1
  address-family ipv4 unicast
   use af-group GROUP_3
   weight 200
  !
 !
!
";

    /// Doc sections "Configuring a Route Reflector for BGP" and "Adjusting BGP Timers".
    const ROUTE_REFLECTOR: &str = "
router bgp 120
 timers bgp 30 90
 bgp default local-preference 200
 bgp cluster-id 192.168.70.1
 nsr
 address-family ipv4 unicast
 !
 neighbor 172.168.40.24
  remote-as 2003
  timers 60 220
  address-family ipv4 unicast
   route-reflector-client
   soft-reconfiguration inbound always
   maximum-prefix 2500 75 warning-only
  !
 !
 neighbor 172.168.40.25
  remote-as 2003
  shutdown
 !
!
";

    /// Doc section "Configuring BGP as a PE-CE Protocol" (shortened).
    const PE_CE: &str = "
router bgp 120
 vrf vrf_pe_2
  rd 345:567
  bgp router-id 172.16.9.9
  address-family ipv4 unicast
   network 172.16.5.5
   aggregate-address 10.0.0.0/24
  !
  neighbor 10.0.0.0
   remote-as 2
   password encrypted 234xyz
   ebgp-multihop 55
   address-family ipv4 unicast
    site-of-origin 234:111
    as-override
    allowas-in 5
    route-policy pe_ce_in_policy in
    route-policy pe_ce_out_policy out
   !
  !
 !
!
";

    fn process(text: &str) -> BgpProcess {
        let procs = bgp_processes(&cisco::parse(text));
        assert_eq!(procs.len(), 1);
        procs.into_iter().next().unwrap()
    }

    #[test]
    fn enabling_bgp_example() {
        let bgp = process(ENABLING_BGP);
        assert_eq!(bgp.asn, "65000");
        assert_eq!(bgp.router_id.as_deref(), Some("1.1.1.1"));
        assert_eq!(bgp.confederation_identifier.as_deref(), Some("1"));
        assert_eq!(bgp.confederation_peers, ["65001", "65002"]);

        let names: Vec<_> = bgp.address_families.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["ipv4 unicast", "ipv4 multicast"]);
        let unicast = &bgp.address_families[0];
        assert_eq!(
            unicast.aggregate_addresses,
            ["10.2.0.0/24 route-policy set_next_hop_agg_v4", "10.3.0.0/24"]
        );
        assert_eq!(unicast.redistributions, ["static route-policy set_next_hop_static_v4"]);

        let addresses: Vec<_> = bgp.neighbors.iter().map(|n| n.address.as_str()).collect();
        assert_eq!(addresses, ["10.0.101.60", "10.0.101.62", "10.0.101.64"]);
        let ebgp = &bgp.neighbors[2];
        assert_eq!(ebgp.remote_as.as_deref(), Some("5"));
        assert_eq!(ebgp.update_source.as_deref(), Some("Loopback0"));
        let af = &ebgp.address_families[0];
        assert_eq!(af.name, "ipv4 unicast");
        assert_eq!(af.settings.route_policy_in.as_deref(), Some("pass-all"));
        assert_eq!(af.settings.route_policy_out.as_deref(), Some("pass-all"));
    }

    #[test]
    fn template_groups_and_inheritance() {
        let bgp = process(TEMPLATES);

        let af_group = &bgp.af_groups[0];
        assert_eq!(
            (af_group.name.as_str(), af_group.address_family.as_str()),
            ("GROUP_3", "ipv4 unicast")
        );
        assert!(af_group.settings.next_hop_self);
        assert_eq!(af_group.settings.route_policy_in.as_deref(), Some("POLICY_1"));

        let session_group = &bgp.session_groups[0];
        assert_eq!(session_group.name, "GROUP_2");
        assert_eq!(session_group.advertisement_interval, Some(15));

        let group = &bgp.neighbor_groups[0];
        assert_eq!(group.name, "GROUP_1");
        assert_eq!(group.use_session_group.as_deref(), Some("GROUP_2"));
        assert_eq!(group.ebgp_multihop, Some(3));
        assert_eq!(group.address_families.len(), 2);
        assert_eq!(group.address_families[0].settings.weight, Some(100));
        assert!(group.address_families[0].settings.send_community_ebgp);
        assert!(group.address_families[1].settings.default_originate);

        let neighbor = &bgp.neighbors[0];
        assert_eq!(neighbor.use_neighbor_group.as_deref(), Some("GROUP_1"));
        assert_eq!(
            neighbor.address_families[0].settings.use_af_group.as_deref(),
            Some("GROUP_3")
        );
        assert_eq!(neighbor.address_families[0].settings.weight, Some(200));
    }

    #[test]
    fn route_reflector_timers_and_process_settings() {
        let bgp = process(ROUTE_REFLECTOR);
        assert_eq!(
            bgp.timers,
            Some(BgpTimers {
                keepalive: 30,
                hold_time: 90
            })
        );
        assert_eq!(bgp.default_local_preference, Some(200));
        assert_eq!(bgp.cluster_id.as_deref(), Some("192.168.70.1"));
        assert!(bgp.nsr);

        let client = &bgp.neighbors[0];
        assert_eq!(
            client.timers,
            Some(BgpTimers {
                keepalive: 60,
                hold_time: 220
            })
        );
        assert!(!client.shutdown);
        let af = &client.address_families[0].settings;
        assert!(af.route_reflector_client);
        assert!(af.soft_reconfiguration_inbound);
        assert_eq!(af.maximum_prefix, Some(2500));

        assert!(bgp.neighbors[1].shutdown);
    }

    #[test]
    fn vrf_pe_ce_neighbor() {
        let bgp = process(PE_CE);
        let vrf = &bgp.vrfs[0];
        assert_eq!(vrf.name, "vrf_pe_2");
        assert_eq!(vrf.rd.as_deref(), Some("345:567"));
        assert_eq!(vrf.router_id.as_deref(), Some("172.16.9.9"));
        assert_eq!(vrf.address_families[0].networks, ["172.16.5.5"]);
        assert_eq!(vrf.address_families[0].aggregate_addresses, ["10.0.0.0/24"]);

        let ce = &vrf.neighbors[0];
        assert_eq!(ce.address, "10.0.0.0");
        assert_eq!(ce.ebgp_multihop, Some(55));
        let af = &ce.address_families[0].settings;
        assert_eq!(af.site_of_origin.as_deref(), Some("234:111"));
        assert!(af.as_override);
        assert_eq!(af.allowas_in, Some(5));
        assert_eq!(af.route_policy_in.as_deref(), Some("pe_ce_in_policy"));
        assert_eq!(af.route_policy_out.as_deref(), Some("pe_ce_out_policy"));
        assert!(bgp.neighbors.is_empty());
    }

    #[test]
    fn non_bgp_block_is_none() {
        let config = cisco::parse("router ospf 1\n router-id 1.1.1.1\n");
        assert!(BgpProcess::from_block(&config.blocks[0]).is_none());
    }
}
