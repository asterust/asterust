//! `protocols { ospf { ... } }`.

use crate::{arg_of, child_block, has_leaf, named_children, top_level};
use asterust::config::Config;

/// An `interface <name>` entry under an OSPF area.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfInterface {
    pub name: String,
    pub passive: bool,
    pub metric: Option<u32>,
}

/// An `area <id>` and its interfaces.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfArea {
    pub id: String,
    pub interfaces: Vec<OspfInterface>,
}

/// The `protocols ospf` configuration. `ospf3` is not covered.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Ospf {
    pub areas: Vec<OspfArea>,
}

/// The `protocols ospf` block, or `None` if OSPF is not configured.
pub fn ospf(config: &Config) -> Option<Ospf> {
    let ospf = child_block(top_level(config, "protocols")?, "ospf")?;
    let areas = named_children(ospf, "area")
        .map(|(id, area)| OspfArea {
            id: id.to_string(),
            interfaces: named_children(area, "interface")
                .map(|(name, iface)| OspfInterface {
                    name: name.to_string(),
                    passive: has_leaf(iface, "passive"),
                    metric: arg_of(iface, "metric").and_then(|m| m.parse().ok()),
                })
                .collect(),
        })
        .collect();
    Some(Ospf { areas })
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::juniper;

    // Not from documentation: covers statements the Juniper examples do not use. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    ospf {
        area 0.0.0.0 {
            interface ge-0/0/0.0 {
                metric 10;
            }
            interface lo0.0 {
                passive;
            }
        }
        area 0.0.0.10 {
            interface ge-0/0/1.0;
        }
    }
    ospf3 {
        area 0.0.0.0 {
            interface ge-0/0/0.0;
        }
    }
}
"#;

    #[test]
    fn extracts_areas_and_interfaces() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        assert_eq!(ospf.areas.len(), 2);

        let backbone = &ospf.areas[0];
        assert_eq!(backbone.id, "0.0.0.0");
        assert_eq!(backbone.interfaces.len(), 2);
        assert_eq!(backbone.interfaces[0].name, "ge-0/0/0.0");
        assert_eq!(backbone.interfaces[0].metric, Some(10));
        assert!(!backbone.interfaces[0].passive);
        assert!(backbone.interfaces[1].passive);

        let area10 = &ospf.areas[1];
        assert_eq!(area10.id, "0.0.0.10");
        assert_eq!(area10.interfaces[0].name, "ge-0/0/1.0");
    }

    #[test]
    fn no_ospf_is_none() {
        assert!(ospf(&juniper::parse_braces("protocols {\n isis {\n }\n}\n")).is_none());
    }

    // Adapted from Juniper's BGP User Guide, "BGP Peering Sessions", "Example: Configuring Internal BGP Peer
    // Sessions", Device A `show protocols` (OSPF part); see the README "Sources".
    const DOC_OSPF: &str = r#"
protocols {
    ospf {
        area 0.0.0.0 {
            interface lo0.1 {
                passive;
            }
            interface ge-0/1/0.1;
        }
    }
}
"#;

    #[test]
    fn doc_area_with_passive_loopback() {
        let ospf = ospf(&juniper::parse_braces(DOC_OSPF)).unwrap();
        assert_eq!(ospf.areas.len(), 1);
        let area = &ospf.areas[0];
        assert_eq!(area.id, "0.0.0.0");
        assert_eq!(area.interfaces[0].name, "lo0.1");
        assert!(area.interfaces[0].passive);
        assert_eq!(area.interfaces[1].name, "ge-0/1/0.1");
        assert!(!area.interfaces[1].passive);
    }
}
