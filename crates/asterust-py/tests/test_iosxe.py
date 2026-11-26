from asterust import parse_cisco
from asterust.iosxe import IosXeDevice

RUNNING_CONFIG = """
interface Loopback0
 ip address 10.255.0.1 255.255.255.255
 ip ospf 1 area 0
 ip router isis CORE
!
interface GigabitEthernet0/0/0
 description to p1
 vrf forwarding CUSTOMER-A
 ip address 10.0.1.1 255.255.255.252
 ipv6 address 2001:db8:1::1/64
 shutdown
!
router isis CORE
 net 49.0001.0100.2550.0001.00
!
router isis
 net 49.0002.0100.2550.0001.00
!
router ospf 1
 router-id 10.255.0.1
 passive-interface default
 network 10.0.1.0 0.0.0.3 area 0
!
router bgp 65000
 bgp router-id 10.255.0.1
 neighbor RR peer-group
 neighbor 10.255.0.2 remote-as 65000
 neighbor 10.255.0.2 description rr1
 neighbor 10.255.0.3 peer-group RR
 !
 address-family ipv4
  neighbor 10.255.0.2 activate
 exit-address-family
!
"""


def test_interfaces():
    device = IosXeDevice(RUNNING_CONFIG)
    loopback, gig = device.interfaces

    assert loopback.name == "Loopback0"
    assert (loopback.ospf_process_id, loopback.ospf_area) == ("1", "0")
    assert loopback.isis_tag == "CORE"

    assert gig.description == "to p1"
    assert gig.vrf == "CUSTOMER-A"
    assert gig.ipv4_addresses == ["10.0.1.1 255.255.255.252"]
    assert gig.ipv6_addresses == ["2001:db8:1::1/64"]
    assert gig.shutdown is True
    assert gig.ospf_area is None


def test_ospf_bgp_isis():
    device = IosXeDevice(RUNNING_CONFIG)

    ospf = device.ospf[0]
    assert (ospf.process_id, ospf.vrf, ospf.router_id) == ("1", None, "10.255.0.1")
    assert ospf.passive_interfaces == ["default"]
    assert (ospf.networks[0].prefix, ospf.networks[0].wildcard, ospf.networks[0].area) == (
        "10.0.1.0",
        "0.0.0.3",
        "0",
    )

    bgp = device.bgp[0]
    assert bgp.asn == "65000"
    assert bgp.address_families == ["ipv4"]
    assert [n.address for n in bgp.neighbors] == ["10.255.0.2", "10.255.0.3"]
    assert bgp.neighbors[0].remote_as == "65000"
    assert bgp.neighbors[0].description == "rr1"
    assert bgp.neighbors[1].peer_group == "RR"

    assert [i.tag for i in device.isis] == ["CORE", None]
    assert device.isis[0].nets == ["49.0001.0100.2550.0001.00"]


def test_from_config_matches_text_constructor():
    from_text = IosXeDevice(RUNNING_CONFIG)
    from_tree = IosXeDevice.from_config(parse_cisco(RUNNING_CONFIG))
    assert from_tree.interfaces == from_text.interfaces
    assert len(from_tree.config) == len(from_text.config)


def test_repr_uses_python_class_name():
    interface = IosXeDevice(RUNNING_CONFIG).interfaces[0]
    assert repr(interface).startswith("IosXeInterface")
    assert "Loopback0" in repr(interface)


# Written from the IOS XE command syntax; not checked against a Cisco guide or a device (see the asterust-iosxe README).
VRF_CONFIG = """
vrf definition CUSTOMER-A
 description Customer A
 rd 65000:1
 !
 address-family ipv4
  route-target export 65000:1
  route-target both 65000:100
  import map FROM-CORE
 exit-address-family
!
ip vrf LEGACY
 rd 65000:9
 route-target import 65000:9
!
"""


def test_vrfs():
    new_style, legacy = IosXeDevice(VRF_CONFIG).vrfs

    assert (new_style.name, new_style.rd, new_style.legacy) == ("CUSTOMER-A", "65000:1", False)
    assert new_style.description == "Customer A"
    af = new_style.address_families[0]
    assert af.name == "ipv4"
    assert af.export_route_targets == ["65000:1", "65000:100"]
    assert af.import_route_targets == ["65000:100"]
    assert af.import_map == "FROM-CORE"

    assert legacy.legacy is True
    assert legacy.address_families[0].import_route_targets == ["65000:9"]
