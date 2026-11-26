use asterust_iosxe::IosXeDevice;

const RUNNING_CONFIG: &str = "
Building configuration...

Current configuration : 1234 bytes
!
version 17.9
hostname pe1
!
vrf definition CUSTOMER-A
 rd 65000:1
!
interface Loopback0
 ip address 10.255.0.1 255.255.255.255
 ip ospf 1 area 0
!
interface GigabitEthernet0/0/0
 description to p1
 ip address 10.0.1.1 255.255.255.252
 ip router isis CORE
 negotiation auto
!
router isis CORE
 net 49.0001.0100.2550.0001.00
!
router ospf 1
 router-id 10.255.0.1
 network 10.0.1.0 0.0.0.3 area 0
!
router bgp 65000
 bgp router-id 10.255.0.1
 neighbor 10.255.0.2 remote-as 65000
 !
 address-family ipv4
  neighbor 10.255.0.2 activate
 exit-address-family
!
end
";

#[test]
fn extracts_every_supported_block_from_one_config() {
    let device = IosXeDevice::parse(RUNNING_CONFIG);

    let interfaces = device.interfaces();
    assert_eq!(interfaces.len(), 2);
    assert_eq!(interfaces[0].ospf_area.as_deref(), Some("0"));
    assert_eq!(interfaces[1].isis_tag.as_deref(), Some("CORE"));
    assert_eq!(device.isis()[0].tag.as_deref(), Some("CORE"));
    assert_eq!(device.ospf()[0].networks[0].area, "0");
    assert_eq!(device.bgp()[0].neighbors[0].address, "10.255.0.2");
}
