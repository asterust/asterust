//! `topologies`
//! `protocols { isis { topologies { ipv4-multicast; ipv6-multicast; ipv6-unicast; } } }`.
//!
//! # See also
//!
//! - [topologies (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/topologies-edit-protocols-isis.html)

use crate::statement_words;
use asterust::config::ConfigLine;

/// The IS-IS `topologies` statement, which enables the extra SPF topologies.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct IsisTopologies {
    pub ipv4_multicast: bool,
    pub ipv6_multicast: bool,
    pub ipv6_unicast: bool,
}

pub(super) fn parse_topologies(isis: &ConfigLine) -> Option<IsisTopologies> {
    let words = statement_words(isis, "topologies")?;
    Some(IsisTopologies {
        ipv4_multicast: words.contains(&"ipv4-multicast"),
        ipv6_multicast: words.contains(&"ipv6-multicast"),
        ipv6_unicast: words.contains(&"ipv6-unicast"),
    })
}

#[cfg(test)]
mod tests {
    use crate::protocols::isis::isis;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const BLOCK: &str = r#"
protocols {
    isis {
        topologies {
            ipv4-multicast;
            ipv6-unicast;
        }
    }
}
"#;

    #[test]
    fn reads_the_block_form() {
        let topologies = isis(&juniper::parse_braces(BLOCK)).unwrap().topologies.unwrap();
        assert!(topologies.ipv4_multicast);
        assert!(topologies.ipv6_unicast);
        assert!(!topologies.ipv6_multicast);
    }

    #[test]
    fn reads_the_one_line_form() {
        let config = juniper::parse_braces("protocols {\n    isis {\n        topologies ipv6-unicast;\n    }\n}\n");
        let topologies = isis(&config).unwrap().topologies.unwrap();
        assert!(topologies.ipv6_unicast);
        assert!(!topologies.ipv4_multicast);
    }
}
