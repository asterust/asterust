# asterust-iosxe

Typed views of common Cisco IOS XE config blocks, extracted from a config
tree parsed by [asterust](https://github.com/asterust/asterust).

| Module | Block | Extracted |
| --- | --- | --- |
| `interface` | `interface <name>` | description, vrf, ipv4/ipv6 addresses, shutdown, per-interface OSPF area and IS-IS tag |
| `ospf` | `router ospf <id> [vrf <name>]` | router-id, `network` statements, passive interfaces |
| `bgp` | `router bgp <asn>` | router-id, address-families, neighbors (remote-as, description, peer-group) |
| `isis` | `router isis [tag]` | net |
| `vrf` | `vrf definition <name>`, `ip vrf <name>` | description, rd, per-address-family route-targets and import/export maps |

```rust
use asterust_iosxe::IosXeDevice;

let device = IosXeDevice::parse(running_config_text);
for iface in device.interfaces() {
    println!("{}: {:?}", iface.name, iface.ipv4_addresses);
}
```

The per-module functions (e.g. `interface::interfaces(&config)`) work on any
`asterust::config::Config` if you already have one parsed.

## Sources

Unlike `asterust-iosxr`, these views are not yet grounded in a Cisco guide: the test fixtures are written from the IOS XE command syntax and have not been checked against a Cisco guide or a real device. Treat the `vrf` fixtures in particular as unverified.
