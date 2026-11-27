//! `post-convergence-lfa`
//! `protocols { ospf { area <id> { interface <name> { post-convergence-lfa [node-protection]; } } } }`.
//!
//! # See also
//!
//! - [post-convergence-lfa (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/post-convergence-lfa-edit-protocols-ospf.html)
//! - [Configuring Segment Routing Microloop Avoidance in OSPFv2 Networks](https://networkingdocs.hpe.com/documentation/us/en/software/junos/segment-routing/topics/example/ospfv2-microloop-avoidance-configuring.html)

use crate::{statement_words, value_after};
use asterust::config::ConfigLine;

/// The `post-convergence-lfa` statement of an interface (TI-LFA). A bare `post-convergence-lfa;` leaves every
/// field unset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfPostConvergenceLfa {
    pub node_protection: bool,
    /// From `node-protection cost <cost>`.
    pub cost: Option<String>,
    pub fate_sharing_protection: bool,
    pub srlg_protection: bool,
}

pub(super) fn parse_post_convergence_lfa(iface: &ConfigLine) -> Option<OspfPostConvergenceLfa> {
    let words = statement_words(iface, "post-convergence-lfa")?;
    Some(OspfPostConvergenceLfa {
        node_protection: words.contains(&"node-protection"),
        cost: value_after(&words, "cost"),
        fate_sharing_protection: words.contains(&"fate-sharing-protection"),
        srlg_protection: words.contains(&"srlg-protection"),
    })
}

#[cfg(test)]
mod tests {
    use crate::protocols::ospf::ospf;
    use asterust::config::juniper;

    // Adapted from "Configuring Segment Routing Microloop Avoidance in OSPFv2 Networks" (Segment Routing User
    // Guide), Device R0 `show protocols` (OSPF part), plus a variant written from the CLI Reference; see the README
    // "Sources".
    const CONFIG: &str = r#"
protocols {
    ospf {
        area 0.0.0.0 {
            interface xe-0/0/0:0.0 {
                interface-type p2p;
                metric 10;
                post-convergence-lfa;
            }
            interface xe-0/0/0:3.0 {
                post-convergence-lfa node-protection cost 20 srlg-protection;
            }
            interface xe-0/0/1:0.0;
        }
    }
}
"#;

    #[test]
    fn reads_the_bare_form() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let lfa = ospf.areas[0].interfaces[0].post_convergence_lfa.as_ref().unwrap();
        assert!(!lfa.node_protection);
        assert_eq!(lfa.cost, None);
    }

    #[test]
    fn reads_the_options() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let lfa = ospf.areas[0].interfaces[1].post_convergence_lfa.as_ref().unwrap();
        assert!(lfa.node_protection);
        assert_eq!(lfa.cost.as_deref(), Some("20"));
        assert!(lfa.srlg_protection);
        assert!(!lfa.fate_sharing_protection);
        assert_eq!(ospf.areas[0].interfaces[2].post_convergence_lfa, None);
    }
}
