//! `protocols { isis { ... } }`.

use crate::{child_block, has_leaf, named_children, top_level};
use asterust::config::Config;

/// An `interface <name>` entry under IS-IS.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct IsisInterface {
    pub name: String,
    pub passive: bool,
    pub point_to_point: bool,
}

/// The `protocols isis` configuration. The NET lives on the loopback's `family iso` address, see
/// [`crate::interface::InterfaceUnit::iso_addresses`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Isis {
    pub interfaces: Vec<IsisInterface>,
}

/// The `protocols isis` block, or `None` if IS-IS is not configured.
pub fn isis(config: &Config) -> Option<Isis> {
    let isis = child_block(top_level(config, "protocols")?, "isis")?;
    Some(Isis {
        interfaces: named_children(isis, "interface")
            .map(|(name, iface)| IsisInterface {
                name: name.to_string(),
                passive: has_leaf(iface, "passive"),
                point_to_point: has_leaf(iface, "point-to-point"),
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use asterust::config::juniper;

    // Written from JunOS hierarchy knowledge, not adapted from documentation; see the README "Sources".
    const CONFIG: &str = r#"
protocols {
    isis {
        level 2 wide-metrics-only;
        interface ge-0/0/0.0 {
            point-to-point;
            level 1 disable;
        }
        interface lo0.0 {
            passive;
        }
        interface ge-0/0/1.0;
    }
}
"#;

    #[test]
    fn extracts_interfaces() {
        let isis = isis(&juniper::parse_braces(CONFIG)).unwrap();
        let names: Vec<_> = isis.interfaces.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, ["ge-0/0/0.0", "lo0.0", "ge-0/0/1.0"]);
        assert!(isis.interfaces[0].point_to_point);
        assert!(!isis.interfaces[0].passive);
        assert!(isis.interfaces[1].passive);
        assert!(!isis.interfaces[2].point_to_point);
    }

    #[test]
    fn no_isis_is_none() {
        assert!(isis(&juniper::parse_braces("protocols {\n bgp {\n }\n}\n")).is_none());
    }
}
