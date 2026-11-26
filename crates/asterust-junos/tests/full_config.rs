use asterust::device::Device;
use asterust_junos::JunosDevice;

// Written from JunOS hierarchy knowledge, not adapted from documentation; see the crate README "Sources".
const CONFIG: &str = r#"
system {
    host-name pe1;
    name-server {
        192.0.2.53;
    }
    ntp {
        server 192.0.2.10 prefer;
    }
}
interfaces {
    ge-0/0/0 {
        description "to p1";
        unit 0 {
            family inet {
                address 10.0.1.1/30;
            }
            family iso;
        }
    }
    lo0 {
        unit 0 {
            family inet {
                address 10.255.0.1/32;
            }
            family iso {
                address 49.0001.0100.2550.0001.00;
            }
        }
    }
}
routing-options {
    router-id 10.255.0.1;
    autonomous-system 65000;
}
protocols {
    isis {
        interface ge-0/0/0.0 {
            point-to-point;
        }
        interface lo0.0 {
            passive;
        }
    }
    ospf {
        area 0.0.0.0 {
            interface ge-0/0/0.0;
        }
    }
    bgp {
        group IBGP {
            type internal;
            local-address 10.255.0.1;
            neighbor 10.255.0.2;
        }
    }
}
routing-instances {
    CUSTOMER-A {
        instance-type vrf;
        route-distinguisher 65000:1;
        vrf-target target:65000:1;
    }
}
policy-options {
    policy-statement EXPORT-LOOPBACK {
        from protocol direct;
        then accept;
    }
    community EDGE members 65000:100;
}
firewall {
    family inet {
        filter PROTECT-RE {
            term SSH {
                from destination-port ssh;
                then accept;
            }
        }
    }
}
"#;

#[test]
fn extracts_every_supported_view_from_one_config() {
    let device = JunosDevice::parse(CONFIG);

    assert_eq!(device.interfaces().len(), 2);
    assert_eq!(
        device.routing_options().unwrap().autonomous_system.as_deref(),
        Some("65000")
    );
    assert_eq!(device.isis().unwrap().interfaces.len(), 2);
    assert_eq!(device.ospf().unwrap().areas[0].id, "0.0.0.0");
    assert_eq!(device.bgp().unwrap().groups[0].neighbors[0].address, "10.255.0.2");
    assert!(device.routing_instances()[0].is_vrf());
    let policies = device.policy_options().unwrap();
    assert_eq!(policies.policy_statements[0].name, "EXPORT-LOOPBACK");
    assert_eq!(policies.communities[0].members, ["65000:100"]);
    assert_eq!(device.firewall().unwrap().filters[0].name, "PROTECT-RE");
    assert_eq!(device.system().unwrap().host_name.as_deref(), Some("pe1"));
    assert_eq!(device.system().unwrap().name_servers[0].address, "192.0.2.53");
    assert!(device.system().unwrap().ntp.unwrap().servers[0].prefer);
}

#[test]
fn usable_through_the_device_trait() {
    fn block_count<D: Device>(text: &str) -> usize {
        D::parse(text).config().get_block_count()
    }
    assert_eq!(block_count::<JunosDevice>(CONFIG), 7);
}
