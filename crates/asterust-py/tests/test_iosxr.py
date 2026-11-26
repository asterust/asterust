from asterust import parse_cisco
from asterust.iosxr import IosXrDevice

RUNNING_CONFIG = """
interface Loopback0
 ipv4 address 10.255.0.1 255.255.255.255
!
interface TenGigE0/0/0/1
 description to p1
 vrf CUSTOMER-A
 ipv4 address 10.0.1.1 255.255.255.252
 ipv6 address 2001:db8:1::1/64
 shutdown
!
router isis CORE
 net 49.0001.0100.2550.0001.00
 interface Loopback0
 !
!
router ospf 1
 router-id 10.255.0.1
 area 0
  interface TenGigE0/0/0/1
  !
 !
!
router bgp 65000
 bgp router-id 10.255.0.1
 address-family ipv4 unicast
 !
 neighbor 10.255.0.2
  remote-as 65000
  description rr1
 !
!
"""


def test_interfaces():
    device = IosXrDevice(RUNNING_CONFIG)
    names = [i.name for i in device.interfaces]
    assert names == ["Loopback0", "TenGigE0/0/0/1"]

    ten_gig = device.interfaces[1]
    assert ten_gig.description == "to p1"
    assert ten_gig.vrf == "CUSTOMER-A"
    assert ten_gig.ipv4_addresses == ["10.0.1.1 255.255.255.252"]
    assert ten_gig.ipv6_addresses == ["2001:db8:1::1/64"]
    assert ten_gig.shutdown is True
    assert device.interfaces[0].shutdown is False


def test_ospf_bgp_isis():
    device = IosXrDevice(RUNNING_CONFIG)

    ospf = device.ospf[0]
    assert (ospf.process_id, ospf.router_id) == ("1", "10.255.0.1")
    assert ospf.areas[0].id == "0"
    assert ospf.areas[0].interfaces == ["TenGigE0/0/0/1"]

    bgp = device.bgp[0]
    assert bgp.asn == "65000"
    assert [af.name for af in bgp.address_families] == ["ipv4 unicast"]
    assert bgp.neighbors[0].address == "10.255.0.2"
    assert bgp.neighbors[0].remote_as == "65000"
    assert bgp.neighbors[0].description == "rr1"

    isis = device.isis[0]
    assert isis.tag == "CORE"
    assert isis.nets == ["49.0001.0100.2550.0001.00"]
    assert isis.interfaces == ["Loopback0"]


def test_from_config_matches_text_constructor():
    from_text = IosXrDevice(RUNNING_CONFIG)
    from_tree = IosXrDevice.from_config(parse_cisco(RUNNING_CONFIG))
    assert from_tree.interfaces == from_text.interfaces
    assert len(from_tree.config) == len(from_text.config)


def test_data_classes_are_frozen_and_have_repr():
    interface = IosXrDevice(RUNNING_CONFIG).interfaces[0]
    assert "Loopback0" in repr(interface)
    try:
        interface.name = "other"
    except AttributeError:
        return
    raise AssertionError("expected frozen attribute")


# Adapted from Cisco's "Implementing BGP on Cisco IOS XR Software" (see the asterust-iosxr README).
BGP_TEMPLATES = """
router bgp 142
 timers bgp 30 90
 bgp confederation identifier 1
 bgp confederation peers
  65001
  65002
 !
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
 !
 neighbor 192.168.0.1
  remote-as 2
  timers 60 220
  use neighbor-group GROUP_1
  address-family ipv4 unicast
   use af-group GROUP_3
   weight 200
  !
 !
 vrf vrf_pe_2
  rd 345:567
  neighbor 10.0.0.0
   remote-as 2
   address-family ipv4 unicast
    as-override
    allowas-in 5
   !
  !
 !
!
"""


def test_bgp_templates_inheritance_and_vrfs():
    bgp = IosXrDevice(BGP_TEMPLATES).bgp[0]
    assert (bgp.timers.keepalive, bgp.timers.hold_time) == (30, 90)
    assert bgp.confederation_identifier == "1"
    assert bgp.confederation_peers == ["65001", "65002"]

    af_group = bgp.af_groups[0]
    assert (af_group.name, af_group.address_family) == ("GROUP_3", "ipv4 unicast")
    assert af_group.next_hop_self is True
    assert af_group.route_policy_in == "POLICY_1"

    assert bgp.session_groups[0].advertisement_interval == 15

    group = bgp.neighbor_groups[0]
    assert group.use_session_group == "GROUP_2"
    assert group.ebgp_multihop == 3
    assert group.address_families[0].weight == 100
    assert group.address_families[0].send_community_ebgp is True

    neighbor = bgp.neighbors[0]
    assert neighbor.remote_as == "2"
    assert (neighbor.timers.keepalive, neighbor.timers.hold_time) == (60, 220)
    assert neighbor.use_neighbor_group == "GROUP_1"
    assert neighbor.address_families[0].use_af_group == "GROUP_3"
    assert neighbor.address_families[0].weight == 200

    vrf = bgp.vrfs[0]
    assert (vrf.name, vrf.rd) == ("vrf_pe_2", "345:567")
    ce = vrf.neighbors[0]
    assert ce.address == "10.0.0.0"
    assert ce.address_families[0].as_override is True
    assert ce.address_families[0].allowas_in == 5


# Adapted from the same guide, section "Defining the Virtual Routing and Forwarding Tables in Provider Edge Routers".
VRF_CONFIG = """
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
"""


def test_vrfs():
    (vrf,) = IosXrDevice(VRF_CONFIG).vrfs
    assert vrf.name == "vrf_pe"
    af = vrf.address_families[0]
    assert af.name == "ipv4 unicast"
    assert af.maximum_prefix == 2300
    assert (af.import_route_policy, af.export_route_policy) == ("policy_a", "policy_b")
    assert af.import_route_targets == ["234:222"]
    assert af.export_route_targets == ["123:234"]
