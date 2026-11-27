//! `virtual-link`
//! `protocols { ospf { area <id> { virtual-link neighbor-id <router-id> transit-area <area-id> { ... } } } }`.
//!
//! # See also
//!
//! - [virtual-link](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/virtual-link-edit-protocols-ospf.html)
//! - [authentication (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/authentication-edit-protocols-ospf.html)

use crate::protocols::ospf::authentication::{parse_authentication, OspfAuthentication};
use crate::{arg_of, has_leaf, value_after};
use asterust::config::ConfigLine;

/// A `virtual-link` of the backbone area. Values are kept as written.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfVirtualLink {
    /// The router ID of the other end.
    pub neighbor_id: String,
    pub transit_area: String,
    pub disable: bool,
    /// Seconds.
    pub hello_interval: Option<String>,
    /// Seconds.
    pub dead_interval: Option<String>,
    /// Seconds.
    pub retransmit_interval: Option<String>,
    /// Seconds.
    pub transit_delay: Option<String>,
    pub ipsec_sa: Option<String>,
    pub authentication: Option<OspfAuthentication>,
    /// A `keychain` written directly under the link (24.2 and later).
    pub keychain: Option<String>,
}

/// `header` is the text after `virtual-link`, e.g. `neighbor-id 192.0.2.3 transit-area 0.0.0.2`.
pub(super) fn parse_virtual_link(header: &str, link: &ConfigLine) -> OspfVirtualLink {
    let words: Vec<&str> = header.split_whitespace().collect();
    let field = |keyword: &str| arg_of(link, keyword).map(str::to_string);
    OspfVirtualLink {
        neighbor_id: value_after(&words, "neighbor-id").unwrap_or_default(),
        transit_area: value_after(&words, "transit-area").unwrap_or_default(),
        disable: has_leaf(link, "disable"),
        hello_interval: field("hello-interval"),
        dead_interval: field("dead-interval"),
        retransmit_interval: field("retransmit-interval"),
        transit_delay: field("transit-delay"),
        ipsec_sa: field("ipsec-sa"),
        authentication: parse_authentication(link),
        keychain: field("keychain"),
    }
}

#[cfg(test)]
mod tests {
    use crate::protocols::ospf::ospf;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    ospf {
        area 0.0.0.0 {
            virtual-link neighbor-id 192.0.2.3 transit-area 0.0.0.2;
            virtual-link neighbor-id 192.0.2.4 transit-area 0.0.0.3 {
                hello-interval 5;
                dead-interval 20;
                retransmit-interval 3;
                transit-delay 2;
                ipsec-sa SA-VL;
                authentication {
                    md5 1 key "$9$secret"; ## SECRET-DATA
                }
            }
            virtual-link neighbor-id 192.0.2.5 transit-area 0.0.0.4 {
                disable;
                keychain VL-KEYS;
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_header_of_a_link_without_options() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let link = &ospf.areas[0].virtual_links[0];
        assert_eq!(link.neighbor_id, "192.0.2.3");
        assert_eq!(link.transit_area, "0.0.0.2");
        assert!(!link.disable);
        assert_eq!(link.hello_interval, None);
        assert_eq!(link.authentication, None);
    }

    #[test]
    fn reads_the_timers_and_authentication() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let link = &ospf.areas[0].virtual_links[1];
        assert_eq!(link.neighbor_id, "192.0.2.4");
        assert_eq!(link.hello_interval.as_deref(), Some("5"));
        assert_eq!(link.dead_interval.as_deref(), Some("20"));
        assert_eq!(link.retransmit_interval.as_deref(), Some("3"));
        assert_eq!(link.transit_delay.as_deref(), Some("2"));
        assert_eq!(link.ipsec_sa.as_deref(), Some("SA-VL"));
        assert_eq!(link.authentication.as_ref().unwrap().md5[0].id, "1");
    }

    #[test]
    fn reads_disable_and_keychain() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let link = &ospf.areas[0].virtual_links[2];
        assert!(link.disable);
        assert_eq!(link.keychain.as_deref(), Some("VL-KEYS"));
    }
}
