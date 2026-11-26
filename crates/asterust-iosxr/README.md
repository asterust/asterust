# asterust-iosxr

[![CI](https://github.com/asterust/asterust-iosxr/actions/workflows/ci.yml/badge.svg)](https://github.com/asterust/asterust-iosxr/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

Typed views of common Cisco IOS XR config blocks, extracted from a config
tree parsed by [asterust](https://github.com/asterust/asterust).

| Module | Block | Extracted |
| --- | --- | --- |
| `interface` | `interface <name>` | description, vrf, ipv4/ipv6 addresses, shutdown |
| `ospf` | `router ospf <id>` | router-id, areas and their interfaces |
| `bgp` | `router bgp <asn>` | router-id, address-families, neighbors (remote-as, description) |
| `isis` | `router isis <tag>` | net, interfaces |

```rust
use asterust_iosxr::IosXrDevice;

let device = IosXrDevice::parse(running_config_text);
for iface in device.interfaces() {
    println!("{}: {:?}", iface.name, iface.ipv4_addresses);
}
```

The per-module functions (e.g. `interface::interfaces(&config)`) work on any
`asterust::config::Config` if you already have one parsed.
