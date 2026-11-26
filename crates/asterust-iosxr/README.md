# asterust-iosxr

[![CI](https://github.com/asterust/asterust-iosxr/actions/workflows/ci.yml/badge.svg)](https://github.com/asterust/asterust-iosxr/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

Typed views of common Cisco IOS XR config blocks, extracted from a config
tree parsed by [asterust](https://github.com/asterust/asterust).

| Module | Block | Extracted |
| --- | --- | --- |
| `interface` | `interface <name>` | description, vrf, ipv4/ipv6 addresses, shutdown |
| `ospf` | `router ospf <id>` | router-id, areas and their interfaces |
| `bgp` | `router bgp <asn>` | router-id, timers, confederation, route-reflector cluster-id, address-families (networks, aggregates, redistribution), neighbors, neighbor/session/af-group templates, VRFs |
| `isis` | `router isis <tag>` | net, interfaces |
| `vrf` | `vrf <name>` | description, address-families (maximum prefix, import/export route-policy and route-targets) |

```rust
use asterust_iosxr::IosXrDevice;

let device = IosXrDevice::parse(running_config_text);
for iface in device.interfaces() {
    println!("{}: {:?}", iface.name, iface.ipv4_addresses);
}
```

The per-module functions (e.g. `interface::interfaces(&config)`) work on any
`asterust::config::Config` if you already have one parsed.

## Sources

The BGP model and its test fixtures follow Cisco's
[Implementing BGP on Cisco IOS XR Software](https://www.cisco.com/c/en/us/td/docs/routers/xr12000/software/xr12k_r4-1/routing/configuration/guide/routing_cg41xr12k_chapter1.html)
(Cisco IOS XR Routing Configuration Guide for the XR 12000 Series Router, Release 4.1, updated April 2011).

| Test fixture (`src/bgp.rs`) | Documentation section |
| --- | --- |
| `ENABLING_BGP` | Configuration Examples: "Enabling BGP: Example" |
| `TEMPLATES` | BGP Configuration: "Viewing Inherited Configurations" sample configuration |
| `ROUTE_REFLECTOR` | "Configuring a Route Reflector for BGP" and "Adjusting BGP Timers" |
| `PE_CE` | "Configuring BGP as a PE-CE Protocol" |
| `GUIDE_VRF` (`src/vrf.rs`) | "Defining the Virtual Routing and Forwarding Tables in Provider Edge Routers" (the guide's `123;234` route-target is a typo and is written `123:234`) |

The fixtures are adapted, not copied verbatim: the guide mostly shows interactive CLI sessions, so
they are reindented into running-config form and shortened. Release 4.1 is old; the neighbor,
`neighbor-group`, `session-group`, `af-group` and VRF structure is unchanged in current IOS XR, but
check newer syntax against a real device before relying on a command.
