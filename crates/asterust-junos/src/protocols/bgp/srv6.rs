//! `source-packet-routing srv6`
//! `protocols { bgp { source-packet-routing { srv6 { locator <name> { end-dt4-sid <sid>; ... } } } } }`, also valid in a
//! routing instance.
//!
//! # See also
//!
//! - [srv6 (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/srv6-edit-protocols-bgp-source-packet-routing.html)
//! - [advertise-srv6-service](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/advertise-srv6-service-edit-protocols-bgp-family-inet-unicast.html)
//! - [accept-srv6-service](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/accept-srv6-service-edit-protocols-bgp-family-inet-unicast.html)
//! - [SRv6 Network Programming and Layer 3 Services in BGP Networks](https://networkingdocs.hpe.com/documentation/us/en/software/junos/segment-routing/topics/topic-map/bgp-srv6-network-programming.html)

use crate::{active_children, child_block, named_children, split_keyword, split_name};
use asterust::config::ConfigLine;

/// `source-packet-routing srv6` of BGP.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpSrv6 {
    pub locators: Vec<BgpSrv6Locator>,
}

/// A `locator <name>`; its prefix is under `routing-options source-packet-routing srv6`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpSrv6Locator {
    pub name: String,
    pub end_dt4_sid: Option<String>,
    pub end_dt6_sid: Option<String>,
    pub end_dt46_sid: Option<String>,
    /// `micro-dt4-sid`, `micro-dt6-sid` and `micro-dt46-sid`.
    pub micro_sids: Vec<BgpSrv6MicroSid>,
    /// Allocates an extra service SID that a policy selects.
    pub non_default: bool,
}

/// A `micro-dt4-sid`, `micro-dt6-sid` or `micro-dt46-sid`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpSrv6MicroSid {
    /// `dt4`, `dt6` or `dt46`.
    pub kind: String,
    /// The static value; without one BGP allocates a dynamic SID.
    pub value: Option<String>,
}

pub(crate) fn parse_srv6(bgp: &ConfigLine) -> Option<BgpSrv6> {
    let srv6 = child_block(child_block(bgp, "source-packet-routing")?, "srv6")?;
    Some(BgpSrv6 {
        locators: named_children(srv6, "locator")
            .map(|(arg, locator)| parse_locator(arg, locator))
            .collect(),
    })
}

fn parse_locator(arg: &str, locator: &ConfigLine) -> BgpSrv6Locator {
    let (name, inline) = split_name(arg);
    let mut result = BgpSrv6Locator {
        name: name.to_string(),
        end_dt4_sid: None,
        end_dt6_sid: None,
        end_dt46_sid: None,
        micro_sids: Vec::new(),
        non_default: false,
    };
    let statements = std::iter::once(inline).chain(active_children(locator).map(|c| c.text.as_str()));
    for statement in statements.filter(|s| !s.is_empty()) {
        let (keyword, rest) = split_keyword(statement);
        let words: Vec<&str> = rest.split_whitespace().collect();
        let value = words.first().filter(|w| **w != "non-default").map(|w| w.to_string());
        result.non_default |= keyword == "non-default" || words.contains(&"non-default");
        match keyword {
            "end-dt4-sid" => result.end_dt4_sid = value,
            "end-dt6-sid" => result.end_dt6_sid = value,
            "end-dt46-sid" => result.end_dt46_sid = value,
            "micro-dt4-sid" | "micro-dt6-sid" | "micro-dt46-sid" => result.micro_sids.push(BgpSrv6MicroSid {
                kind: keyword
                    .trim_start_matches("micro-")
                    .trim_end_matches("-sid")
                    .to_string(),
                value,
            }),
            _ => {}
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::protocols::bgp::bgp;
    use crate::routing_instances::routing_instances;
    use asterust::config::juniper;

    // Router R0 `show protocols` (BGP part) and `show routing-instances` of "Example: Configuring Layer 3 Services over
    // SRv6 in BGP Networks" (Segment Routing User Guide), shortened; see the README "Sources".
    const DOC_SRV6: &str = r#"
protocols {
    bgp {
        group to-PE-all {
            type internal;
            local-address abcd::128:53:38:52;
            family inet {
                unicast {
                    extended-nexthop;
                    advertise-srv6-service;
                    accept-srv6-service;
                }
            }
            family inet-vpn {
                unicast {
                    extended-nexthop;
                    advertise-srv6-service;
                    accept-srv6-service;
                }
            }
            family inet6 {
                unicast {
                    advertise-srv6-service;
                    accept-srv6-service;
                }
            }
            family inet6-vpn {
                unicast {
                    advertise-srv6-service;
                    accept-srv6-service;
                }
            }
            export adv_global;
            neighbor abcd::128:53:35:39;
        }
        group to-TG-global-v4 {
            type external;
            family inet {
                unicast;
            }
            peer-as 1001;
            neighbor 11.1.1.2;
        }
        source-packet-routing {
            srv6 {
                locator loc1 {
                    end-dt4-sid 3001::2;
                    end-dt6-sid 3001::3;
                }
            }
        }
    }
}
routing-instances {
    vpn1 {
        protocols {
            bgp {
                group to-TG-vpn1-v4 {
                    type external;
                    peer-as 1002;
                    neighbor 11.1.1.6;
                }
                source-packet-routing {
                    srv6 {
                        locator loc1 {
                            end-dt4-sid 3001::4;
                            end-dt6-sid 3001::5;
                        }
                    }
                }
            }
        }
        instance-type vrf;
        interface xe-0/0/0:3.1;
        route-distinguisher 100:1;
        vrf-target target:100:1;
    }
}
"#;

    // Not from documentation: written from the CLI Reference (`srv6`) and the uSID example at srv6.md. See the README
    // "Sources".
    const MICRO_SID: &str = r#"
protocols {
    bgp {
        source-packet-routing {
            srv6 {
                locator LOC0 {
                    micro-dt46-sid;
                }
                locator LOC1 {
                    micro-dt4-sid 0xfc30;
                    micro-dt6-sid non-default;
                    end-dt46-sid 3001::46 non-default;
                }
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_guide_locator() {
        let srv6 = bgp(&juniper::parse_braces(DOC_SRV6)).unwrap().srv6.unwrap();
        assert_eq!(srv6.locators.len(), 1);
        let locator = &srv6.locators[0];
        assert_eq!(locator.name, "loc1");
        assert_eq!(locator.end_dt4_sid.as_deref(), Some("3001::2"));
        assert_eq!(locator.end_dt6_sid.as_deref(), Some("3001::3"));
        assert_eq!(locator.end_dt46_sid, None);
        assert!(locator.micro_sids.is_empty());
        assert!(!locator.non_default);
    }

    #[test]
    fn reads_the_service_families() {
        let bgp = bgp(&juniper::parse_braces(DOC_SRV6)).unwrap();
        let families = &bgp.groups[0].options.address_families;
        assert_eq!(families.len(), 4);
        assert!(families
            .iter()
            .all(|f| f.advertise_srv6_service && f.accept_srv6_service));
        assert!(families[0].extended_nexthop);
        assert!(!families[2].extended_nexthop);

        let plain = &bgp.groups[1].options.address_families[0];
        assert!(!plain.advertise_srv6_service);
        assert!(!plain.accept_srv6_service);
    }

    #[test]
    fn reads_the_routing_instance_locator() {
        let instances = routing_instances(&juniper::parse_braces(DOC_SRV6));
        let srv6 = instances[0].bgp_srv6.as_ref().unwrap();
        assert_eq!(srv6.locators[0].end_dt4_sid.as_deref(), Some("3001::4"));
        assert_eq!(srv6.locators[0].end_dt6_sid.as_deref(), Some("3001::5"));
        assert!(instances[0].is_vrf());
    }

    #[test]
    fn reads_micro_sids() {
        let srv6 = bgp(&juniper::parse_braces(MICRO_SID)).unwrap().srv6.unwrap();
        let dynamic = &srv6.locators[0];
        assert_eq!(dynamic.micro_sids.len(), 1);
        assert_eq!(dynamic.micro_sids[0].kind, "dt46");
        assert_eq!(dynamic.micro_sids[0].value, None);

        let mixed = &srv6.locators[1];
        assert_eq!(mixed.micro_sids[0].kind, "dt4");
        assert_eq!(mixed.micro_sids[0].value.as_deref(), Some("0xfc30"));
        assert_eq!(mixed.micro_sids[1].kind, "dt6");
        assert_eq!(mixed.micro_sids[1].value, None);
        assert_eq!(mixed.end_dt46_sid.as_deref(), Some("3001::46"));
        assert!(mixed.non_default);
    }

    #[test]
    fn no_srv6_is_none() {
        let config = juniper::parse_braces(
            "protocols {\n    bgp {\n        group G {\n            type internal;\n        }\n    }\n}\n",
        );
        assert_eq!(bgp(&config).unwrap().srv6, None);
    }
}
