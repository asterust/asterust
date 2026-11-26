# asterust-junos

Typed views of common Juniper JunOS config statements, extracted from a config tree parsed by
[asterust](https://github.com/asterust/asterust) (`config::juniper`, brace-format `show configuration`).

| Module | Block | Extracted |
| --- | --- | --- |
| `interface` | `interfaces` | description, disable, units (description, vlan-id, families, inet/inet6/iso addresses) |
| `ospf` | `protocols ospf` | areas and their interfaces (passive, metric) |
| `bgp` | `protocols bgp` | groups (type, peer-as, local-as, local-address, cluster, families, import/export policies) and neighbors |
| `isis` | `protocols isis` | interfaces (passive, point-to-point) |
| `routing_instance` | `routing-instances` | instance-type, route-distinguisher, interfaces, vrf-target, vrf-import/vrf-export |
| `routing_options` | `routing-options` | router-id, autonomous-system |

```rust
use asterust_junos::JunosDevice;

let device = JunosDevice::parse(show_configuration_text);
for iface in device.interfaces() {
    println!("{}: {:?}", iface.name, iface.units);
}
if let Some(bgp) = device.bgp() {
    println!("{} BGP groups", bgp.groups.len());
}
```

Unlike the Cisco crates, JunOS singleton protocols are returned as `Option` (`device.ospf()`, `device.bgp()`,
`device.isis()`), and the views read from named top-level blocks rather than one struct per block.
`JunosDevice` also implements `asterust::device::Device`.

Limitations: statements deactivated with `inactive:` are skipped; `| display set` output is not supported;
`ospf3`, protocols nested inside routing instances, `apply-groups` expansion and BGP group-to-neighbor setting
inheritance are not resolved.

## Sources

These views are **not yet grounded in vendor documentation or a real device**. The test fixtures were written
from knowledge of the JunOS configuration hierarchy and are marked as such in each module's tests. Treat the
extracted fields as unverified until they have been compared against Juniper documentation or real output.

The following Juniper documentation pages were intended as references. When this crate was written none could
be read: the juniper.net URLs redirected to `networkingdocs.hpe.com`, which returned HTTP 404, so the fixtures
are **not** adapted from them and the URLs themselves may be wrong.

| Intended reference | URL tried |
| --- | --- |
| Internal BGP peering sessions example (`protocols bgp group`, `neighbor`, `local-address`, `family`) | `https://www.juniper.net/documentation/us/en/software/junos/bgp/topics/example/bgp-ibgp-configuring.html` |
| Multiarea OSPF example (`protocols ospf area`, `interface`, `routing-options router-id`) | `https://www.juniper.net/documentation/us/en/software/junos/ospf/topics/example/ospf-multiarea-configuring.html` |
| Basic Layer 3 VPN example (`routing-instances`, `instance-type vrf`, `route-distinguisher`, `vrf-target`) | `https://www.juniper.net/documentation/us/en/software/junos/vpn-l3/topics/example/vpns-layer-3-basic-configuring.html` |

A working link to Juniper's documentation, or a sanitized `show configuration` snippet from a real device,
would let these fixtures be replaced with grounded ones and this section updated.
