//! Integration tests: exercise only `asterust`'s public API against a
//! realistic JunOS `show configuration` style brace-delimited config.

use asterust::config::juniper;

const SAMPLE_CONFIG: &str = r#"
system {
    host-name router1;
    domain-name example.com;
}
interfaces {
    ge-0/0/0 {
        description "Uplink to core";
        unit 0 {
            family inet {
                address 10.0.0.1/24;
            }
        }
    }
    ge-0/0/1 {
        disable;
    }
}
protocols {
    isis {
        interface ge-0/0/0.0 {
            level 2 metric 10;
        }
    }
    bgp {
        group core {
            type internal;
            neighbor 10.0.0.2 {
                peer-as 65001;
            }
        }
    }
}
"#;

#[test]
fn parses_all_top_level_blocks() {
    let config = juniper::parse_braces(SAMPLE_CONFIG);
    assert_eq!(config.get_block_count(), 3);
}

#[test]
fn reads_flat_leaf_statements_in_system_block() {
    let config = juniper::parse_braces(SAMPLE_CONFIG);
    let system = config
        .find_blocks_by_pattern("system")
        .into_iter()
        .next()
        .expect("system block");
    assert!(system.find_child_with("host-name router1").is_some());
    assert!(system.find_child_with("domain-name example.com").is_some());
}

#[test]
fn navigates_nested_interface_hierarchy() {
    let config = juniper::parse_braces(SAMPLE_CONFIG);
    let interfaces = config
        .find_blocks_by_pattern("interfaces")
        .into_iter()
        .next()
        .expect("interfaces block");

    let address = interfaces.find_line_with(r"address \S+").expect("address line");
    assert_eq!(address.text, "address 10.0.0.1/24");

    let disabled_iface = interfaces.find_child_with("ge-0/0/1").expect("second interface");
    assert!(disabled_iface.has_child_with("disable"));
}

#[test]
fn navigates_nested_protocol_blocks() {
    let config = juniper::parse_braces(SAMPLE_CONFIG);
    let protocols = config
        .find_blocks_by_pattern("protocols")
        .into_iter()
        .next()
        .expect("protocols block");

    let isis_level = protocols.find_line_with(r"level 2").expect("isis level line");
    assert_eq!(isis_level.text, "level 2 metric 10");

    let peer_as = protocols.find_line_with(r"peer-as").expect("bgp peer-as line");
    assert_eq!(peer_as.text, "peer-as 65001");
}
