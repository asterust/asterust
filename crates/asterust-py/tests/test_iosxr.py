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
    assert bgp.address_families == ["ipv4 unicast"]
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
