# asterust-junos

Typed views of common Juniper JunOS config statements, extracted from a config tree parsed by
[asterust](https://github.com/asterust/asterust) (`config::juniper`, brace-format `show configuration`).

| Module | Block | Extracted |
| --- | --- | --- |
| `interfaces` | `interfaces` | description, disable, mtu, speed, encapsulation, vlan-tagging, flexible-vlan-tagging, native-vlan-id, per-unit-scheduler, no-gratuitous-arp-request, hold-time, ether-options (also gigether-options and fastether-options: auto-negotiation, flow-control, loopback, source-filtering, speed, link-mode, fec, 802.3ad bundle membership), aggregated-ether-options (lacp, bfd-liveness-detection, link-speed, minimum-links, flow-control, loopback, source-filtering), optics-options (low-light alarm and warning, wavelength, tx-power, fec, modulation-format, laser-enable, ...), otn-options (fec, rate, tti, signal-degrade, ...), link-degrade-monitor (thresholds, recovery), auto-configure (vlan-ranges and stacked-vlan-ranges with their dynamic profiles; the authentication password is not extracted), units (description, vlan-id, vlan-id-range, vlan-tags, input-vlan-map and output-vlan-map, encapsulation, families, inet/inet6/iso addresses, and per family: mtu, filter, rpf-check, policer, sampling, address options such as primary, preferred, destination, arp vlan-id and vlan-id-list of family bridge, and vrrp-group/vrrp-inet6-group with tracking; the VRRP authentication-key is not extracted) |
| `protocols::ospf` | `protocols ospf` | global options (reference-bandwidth, preferences, export and import policies, rib-group, overload, spf-options including microloop-avoidance, graceful-restart, traffic-engineering, database-protection, backup-spf-options, source-packet-routing, topologies, domain-id, lsa-refresh-interval, prefix-export-limit and flags), areas (stub, nssa, area-range, virtual-link, network-summary policies, label-switched-path, context-identifier) and their interfaces: passive, metric, priority, hello, dead, retransmit and poll intervals, transit delay, interface type, authentication (key ids only, no keys), BFD, LDP synchronization, neighbors, topologies, protection, post-convergence-lfa and the other interface flags |
| `protocols::bgp` | `protocols bgp` | groups (type, peer-as, local-as, local-address, cluster, families, import/export policies) and neighbors, plus the peer options shared by `bgp`, group and neighbor (local-as, multihop, remove-private, authentication algorithm and key-chain, timers, keep, preferences, metric-out and session flags; keys are not extracted), `bfd-liveness-detection` (including strict mode), `graceful-restart` (long-lived helper, forwarding-state-bit and notification flags), `multipath` (including `prioritization`), per-family settings (`family <afi> <safi>`: `prefix-limit`, `accepted-prefix-limit`, `add-path`, `loops`, `aigp`, `rib-group`, `no-validate`, `extended-nexthop`, `extended-nexthop-color`, `extended-nexthop-tunnel`, `entropy-label`, output-queue, route-refresh and withdraw priorities, per-family `graceful-restart` and flags), group `allow`, `peer-as-list` and `dynamic-neighbor` blocks (including `peer-auto-discovery` for BGP unnumbered), and the `bgp`-level `path-selection`, `multipath-build-priority` and `output-queue-priority` |
| `protocols::isis` | `protocols isis` | interfaces: passive, point-to-point, LSP, CSNP and hello options, LDP synchronization, BFD, protection and the other interface flags, plus each `level` entry (metrics, hello and hold timers, priority, hello authentication type and key chain; keys are not extracted) |
| `routing_instances` | `routing-instances` | instance-type, route-distinguisher, interfaces, vrf-target, vrf-import/vrf-export |
| `routing_options` | `routing-options` | router-id, ipv6-router-id, autonomous-system (loops, asdot-notation, independent-domain), confederation, graceful-restart, nonstop-routing, maximum-paths, forwarding-table, interface-routes, resolution, options syslog, static routes (next-hop, qualified-next-hop, discard/reject/receive, next-table, route attributes and defaults), aggregate and generate routes (policy, brief/full, discard, attributes), martians, ribs (per-table static, aggregate, generate, martians), rib-groups, instance-import/instance-export, auto-export, fate-sharing |
| `policy_options` | `policy-options` | policy statements (terms with `from`/`to` conditions and `then` actions), communities, prefix lists, AS paths and AS path groups |
| `firewall` | `firewall` | `family` filters (terms with `from` conditions and `then` actions) and policers |
| `system` | `system` | host-name, domain-name, domain-search, time-zone, static-host-mapping, backup-router, authentication-order, radius-server and tacplus-server (secrets are not extracted), root-authentication (method names only), master-password (iteration count and function), diag-port-authentication and pic-console-authentication (method names only), license (autoupdate URL, key count, renew), location, ports (console, auxiliary), max-configurations-on-flash, max-configuration-rollbacks, no-redirects, default-address-selection, internet-options, commit, processes, scripts, dynamic-profile-options, extensions (JET applications and providers), autoinstallation (server URLs without passwords), schema, pixel-tracing, system flags (no-multicast-echo, auto-snapshot, ...), archival (site URLs without passwords), accounting (events and destination servers), name-servers, ntp (servers, peers, boot-server, source-address, trusted keys; key values are not extracted), syslog (hosts, files, users, console and their facility/severity rules, archive size/files, per-destination options such as match and explicit-priority), login (message, announcement, idle-timeout, deny-sources, password policy, retry-options, classes with access restrictions, users; passwords and keys are not extracted), services (ssh settings including ciphers, macs, key-exchange and authentication switches, telnet, ftp, netconf over ssh, web-management (http, https, interfaces, ports, certificates, session limits), netconf, xnm-clear-text, xnm-ssl, outbound-ssh clients, subscriber-management, static-subscribers, dhcp-local-server groups, local-policy-decision-function statistics, flow-tap-dtcp, service-deployment; secrets are not extracted) |

The source tree mirrors the configuration hierarchy so a statement is easy to find: a folder per top-level block
(`system/`, `protocols/`, `policy_options/`), a file per statement inside it (`system/radius_server.rs`,
`system/root_authentication.rs`), and a folder instead of a file when that statement has many children of its own
(`system/services/ssh.rs`, `system/login/class.rs`, `system/syslog/host.rs`). The types of each level are re-exported
from its parent, e.g. `system::RadiusServer`.

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
| [BGP Peering Sessions](https://networkingdocs.hpe.com/documentation/us/en/software/junos/bgp/topics/topic-map/bgp-peering-sessions.html) | Example: Configuring External BGP Point-to-Point Peer Sessions (Device E) | `protocols::bgp::DOC_EXTERNAL_PEERS`, `interfaces::DOC_POINT_TO_POINT` |
| same page | Example: Configuring Internal BGP Peer Sessions (Device A) | `protocols::bgp::DOC_INTERNAL_PEERS`, `protocols::ospf::DOC_OSPF`, the `routing_options` doc test |
| same page | Example: Configure Multiple Single-Hop EBGP Sessions on Different Links Using the Same IPv6 Link-Local Address (R1) | `protocols::bgp::DOC_LINK_LOCAL` (quoted neighbor names), `interfaces::DOC_LINK_LOCAL` (VLAN units, `primary` address) |
| [BGP Route Reflectors](https://networkingdocs.hpe.com/documentation/us/en/software/junos/bgp/topics/topic-map/bgp-rr.html) | Example: Configuring a Route Reflector (Device A) | `protocols::bgp::DOC_ROUTE_REFLECTOR` (the guide's rendering of this output drops a closing brace, which is restored) |
| [Routing Policies for BGP Communities](https://networkingdocs.hpe.com/documentation/us/en/software/junos/bgp/topics/topic-map/routing-policies-communities.html) | Examples: Redistribute BGP Routes with a Specific Community Tag into IS-IS (Device C), Removes BGP Communities (R1, R2), Based on the Number of BGP Communities (R2) | `policy_options::DOC_COMMUNITY_TAG`, `policy_options::DOC_COMMUNITY_ACTIONS` (the rendered output wraps lines mid-statement, which are rejoined) |
| [TCP Access Restriction for BGP](https://networkingdocs.hpe.com/documentation/us/en/software/junos/bgp/topics/topic-map/tcp_access_restriction.html) | Examples: Configuring a Filter to Block TCP Access to a Port Except from Specified BGP Peers (Device E), Configuring a Filter to Limit TCP Access to a Port Based On a Prefix List | `firewall::DOC_BGP179_PEERS`, `firewall::DOC_BGP179_PREFIX_LIST` (wrapped lines are rejoined) |
| [Configuring Segment Routing Microloop Avoidance in OSPFv2 Networks](https://networkingdocs.hpe.com/documentation/us/en/software/junos/segment-routing/topics/example/ospfv2-microloop-avoidance-configuring.html) (Segment Routing User Guide, read on 2026-11-27) | Device R0 `show protocols`, the OSPF `spf-options`, `backup-spf-options`, `source-packet-routing` and interface `post-convergence-lfa` parts | the `protocols::ospf` tests `reads_microloop_avoidance`, `reads_the_bare_form`, `reads_post_convergence_lfa_and_source_packet_routing` and `reads_the_one_line_forms_of_the_guide_example` (`maximum-labels`, the second interface and the block forms are added from the CLI Reference) |

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

The `system` fixtures (`host-name`, `domain-name`, `domain-search`, `static-host-mapping`, `backup-router`, `authentication-order`, `radius-server`, `tacplus-server`, `location`, `ports`, `max-configurations-on-flash`, `internet-options`, `commit`, `processes`, `scripts`, `dynamic-profile-options`, `extensions`, `autoinstallation`, `schema`, `pixel-tracing`, `archival`, `accounting`, `time-zone`, `root-authentication`, `master-password`, `diag-port-authentication`, `pic-console-authentication`, `license`, `name-server`, `ntp`, `syslog`, `login`, `services`, `outbound-ssh`) follow the syntax in the Junos CLI Reference pages, not an example configuration.
