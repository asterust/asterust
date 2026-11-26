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

The BGP, interface, OSPF and `routing-options` test fixtures (the `DOC_*` constants and `doc_*` tests) are
adapted from Juniper's BGP User Guide, read on 2026-11-26. They are reindented and shortened, not verbatim, and
the guide's typographic quotes are written as plain quotes.

| Page | Example used | Fixtures |
| --- | --- | --- |
| [BGP Peering Sessions](https://networkingdocs.hpe.com/documentation/us/en/software/junos/bgp/topics/topic-map/bgp-peering-sessions.html) | Example: Configuring External BGP Point-to-Point Peer Sessions (Device E) | `bgp::DOC_EXTERNAL_PEERS`, `interface::DOC_POINT_TO_POINT` |
| same page | Example: Configuring Internal BGP Peer Sessions (Device A) | `bgp::DOC_INTERNAL_PEERS`, `ospf::DOC_OSPF`, the `routing_options` doc test |
| same page | Example: Configure Multiple Single-Hop EBGP Sessions on Different Links Using the Same IPv6 Link-Local Address (R1) | `bgp::DOC_LINK_LOCAL` (quoted neighbor names), `interface::DOC_LINK_LOCAL` (VLAN units, `primary` address) |
| [BGP Route Reflectors](https://networkingdocs.hpe.com/documentation/us/en/software/junos/bgp/topics/topic-map/bgp-rr.html) | Example: Configuring a Route Reflector (Device A) | `bgp::DOC_ROUTE_REFLECTOR` (the guide's rendering of this output drops a closing brace, which is restored) |

Other pages consulted but not used for fixtures:
- [Junos OS documentation index](https://networkingdocs.hpe.com/documentation/product/us/en/junos-os/)
- [BGP Configuration Overview](https://networkingdocs.hpe.com/documentation/us/en/software/junos/bgp/topics/task/routing-protocol-bgp-security-configuring.html), which links to the examples above
- [OSPF User Guide](https://networkingdocs.hpe.com/documentation/us/en/software/junos/ospf/index.html), whose topic pages have not been read

**Not yet grounded in documentation or a real device:** `protocols isis`, `routing-instances` (VRFs, `vrf-target`,
`vrf-import`/`vrf-export`), the `inactive:` handling, `disable`, bracketed policy lists (`export [ a b ]`), and
the other extra cases in the tests. Those fixtures are marked "not from documentation" in the code and were
written from knowledge of the JunOS hierarchy, so treat the corresponding fields as unverified. A Juniper
IS-IS or Layer 3 VPN example page, or a sanitized `show configuration` snippet, would let these be replaced.
