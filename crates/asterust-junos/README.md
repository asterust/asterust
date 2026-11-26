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
| `policy_options` | `policy-options` | policy statements (terms with `from`/`to` conditions and `then` actions), communities, prefix lists, AS paths and AS path groups |
| `firewall` | `firewall` | `family` filters (terms with `from` conditions and `then` actions) and policers |
| `system` | `system` | host-name, domain-name, domain-search, time-zone, static-host-mapping, backup-router, authentication-order, radius-server and tacplus-server (secrets are not extracted), root-authentication (method names only), master-password (iteration count and function), diag-port-authentication and pic-console-authentication (method names only), license (autoupdate URL, key count, renew), location, ports (console, auxiliary), max-configurations-on-flash, max-configuration-rollbacks, no-redirects, default-address-selection, internet-options, commit, processes, scripts, system flags (no-multicast-echo, auto-snapshot, ...), archival (site URLs without passwords), accounting (events and destination servers), name-servers, ntp (servers, peers, boot-server, source-address, trusted keys; key values are not extracted), syslog (hosts, files, users, console and their facility/severity rules, archive size/files, per-destination options such as match and explicit-priority), login (message, announcement, idle-timeout, deny-sources, password policy, retry-options, classes with access restrictions, users; passwords and keys are not extracted), services (ssh settings including ciphers, macs, key-exchange and authentication switches, telnet, ftp, netconf over ssh, web-management (http, https, interfaces, ports, certificates, session limits), netconf, xnm-clear-text, xnm-ssl, outbound-ssh clients; secrets are not extracted) |

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
| [Routing Policies for BGP Communities](https://networkingdocs.hpe.com/documentation/us/en/software/junos/bgp/topics/topic-map/routing-policies-communities.html) | Examples: Redistribute BGP Routes with a Specific Community Tag into IS-IS (Device C), Removes BGP Communities (R1, R2), Based on the Number of BGP Communities (R2) | `policy_options::DOC_COMMUNITY_TAG`, `policy_options::DOC_COMMUNITY_ACTIONS` (the rendered output wraps lines mid-statement, which are rejoined) |
| [TCP Access Restriction for BGP](https://networkingdocs.hpe.com/documentation/us/en/software/junos/bgp/topics/topic-map/tcp_access_restriction.html) | Examples: Configuring a Filter to Block TCP Access to a Port Except from Specified BGP Peers (Device E), Configuring a Filter to Limit TCP Access to a Port Based On a Prefix List | `firewall::DOC_BGP179_PEERS`, `firewall::DOC_BGP179_PREFIX_LIST` (wrapped lines are rejoined) |

Other pages consulted but not used for fixtures:
- [Junos OS documentation index](https://networkingdocs.hpe.com/documentation/product/us/en/junos-os/)
- [BGP Configuration Overview](https://networkingdocs.hpe.com/documentation/us/en/software/junos/bgp/topics/task/routing-protocol-bgp-security-configuring.html), which links to the examples above
- [OSPF User Guide](https://networkingdocs.hpe.com/documentation/us/en/software/junos/ospf/index.html), whose topic pages have not been read

**Not yet grounded in documentation or a real device:** `protocols isis`, `routing-instances` (VRFs, `vrf-target`,
`vrf-import`/`vrf-export`), the `inactive:` handling, `disable`, bracketed policy lists (`export [ a b ]`), and
the other extra cases in the tests, including `policy_options` prefix lists, AS paths and AS path groups,
`local-preference`, `next-hop`, `as-path-prepend`, `next term`/`next policy` and block-form communities, and
the `firewall` policers, flag and list match conditions, and the `count`, `policer`, `log`, `syslog`,
`forwarding-class` and `loss-priority` actions. Those fixtures are marked "not from documentation" in the code and were
written from knowledge of the JunOS hierarchy, so treat the corresponding fields as unverified. A Juniper
IS-IS or Layer 3 VPN example page, or a sanitized `show configuration` snippet, would let these be replaced.

The `system` fixtures (`host-name`, `domain-name`, `domain-search`, `static-host-mapping`, `backup-router`, `authentication-order`, `radius-server`, `tacplus-server`, `location`, `ports`, `max-configurations-on-flash`, `internet-options`, `commit`, `processes`, `scripts`, `archival`, `accounting`, `time-zone`, `root-authentication`, `master-password`, `diag-port-authentication`, `pic-console-authentication`, `license`, `name-server`, `ntp`, `syslog`, `login`, `services`, `outbound-ssh`) follow the syntax in the Junos CLI Reference pages, not an example configuration.
