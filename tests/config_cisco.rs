//! Integration tests: exercise only `asterust`'s public API, the way an
//! external consumer (e.g. a Huginn parser) would, against a realistic
//! multi-protocol Cisco IOS-XR style configuration.

use asterust::config::cisco;

const SAMPLE_CONFIG: &str = r#"
hostname router1
!
interface GigabitEthernet0/0/0/0
 description Uplink to core
 ipv4 address 10.0.0.1 255.255.255.0
 no shutdown
!
interface GigabitEthernet0/0/0/1
 shutdown
!
router isis CORE
 net 49.0001.0000.0000.0001.00
 interface GigabitEthernet0/0/0/0
  address-family ipv4 unicast
!
router bgp 65000
 bgp router-id 10.0.0.1
 neighbor 10.0.0.2
  remote-as 65001
!
"#;

#[test]
fn parses_all_top_level_blocks() {
    let config = cisco::parse(SAMPLE_CONFIG);
    assert_eq!(config.get_block_count(), 5);
}

#[test]
fn finds_interface_blocks_by_pattern() {
    let config = cisco::parse(SAMPLE_CONFIG);
    let interfaces = config.find_blocks_by_pattern(r"^interface ");
    assert_eq!(interfaces.len(), 2);
    assert_eq!(interfaces[0].get_root_line().text, "interface GigabitEthernet0/0/0/0");
    assert_eq!(interfaces[1].get_root_line().text, "interface GigabitEthernet0/0/0/1");
}

#[test]
fn reads_interface_configuration_details() {
    let config = cisco::parse(SAMPLE_CONFIG);
    let uplink = config
        .find_blocks_by_pattern(r"GigabitEthernet0/0/0/0")
        .into_iter()
        .next()
        .expect("uplink interface block");

    assert!(uplink.find_child_with("description").is_some());
    assert_eq!(uplink.find_child_with("description").unwrap().text, "description Uplink to core");
    // "no shutdown" contains "shutdown" as a substring, so this must be anchored.
    assert!(uplink.find_child_with("^shutdown$").is_none());

    let shutdown_iface = config
        .find_blocks_by_pattern(r"GigabitEthernet0/0/0/1")
        .into_iter()
        .next()
        .expect("second interface block");
    assert!(shutdown_iface.find_child_with("^shutdown$").is_some());
}

#[test]
fn navigates_nested_isis_hierarchy() {
    let config = cisco::parse(SAMPLE_CONFIG);
    let isis = config
        .find_blocks_by_pattern(r"router isis")
        .into_iter()
        .next()
        .expect("isis block");

    let process_id = isis.get_root_line().match_str_group(r"router isis (\S+)");
    assert_eq!(process_id.as_deref(), Some("CORE"));

    let iface_line = isis.find_line_with(r"interface GigabitEthernet0/0/0/0").expect("isis interface line");
    assert!(iface_line.has_child_with("address-family ipv4"));
}

#[test]
fn navigates_nested_bgp_hierarchy() {
    let config = cisco::parse(SAMPLE_CONFIG);
    let bgp = config.find_blocks_by_pattern(r"router bgp").into_iter().next().expect("bgp block");

    let asn = bgp.get_root_line().match_int_group(r"router bgp (\d+)");
    assert_eq!(asn, Some(65000));

    let neighbor = bgp.find_line_with(r"neighbor 10\.0\.0\.2").expect("neighbor line");
    assert!(neighbor.has_child_with(r"remote-as 65001"));
}
