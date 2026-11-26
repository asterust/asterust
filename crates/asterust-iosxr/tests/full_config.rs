use asterust::config::cisco;
use asterust_iosxr::bgp::bgp_processes;
use asterust_iosxr::interface::interfaces;
use asterust_iosxr::isis::isis_instances;
use asterust_iosxr::ospf::ospf_processes;

const RUNNING_CONFIG: &str = "
!! IOS XR Configuration 7.9.2
hostname pe1
interface Loopback0
 ipv4 address 10.255.0.1 255.255.255.255
!
interface TenGigE0/0/0/1
 description to p1
 ipv4 address 10.0.1.1 255.255.255.252
!
router isis CORE
 net 49.0001.0100.2550.0001.00
 interface Loopback0
  passive
 !
 interface TenGigE0/0/0/1
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
 !
!
end
";

#[test]
fn extracts_every_supported_block_from_one_config() {
    let config = cisco::parse(RUNNING_CONFIG);

    assert_eq!(interfaces(&config).len(), 2);
    assert_eq!(isis_instances(&config)[0].tag, "CORE");
    assert_eq!(ospf_processes(&config)[0].areas[0].interfaces, ["TenGigE0/0/0/1"]);
    assert_eq!(bgp_processes(&config)[0].neighbors[0].address, "10.255.0.2");
}
