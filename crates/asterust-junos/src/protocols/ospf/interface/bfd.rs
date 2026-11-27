//! `bfd-liveness-detection`
//! `protocols { ospf { area <id> { interface <name> { bfd-liveness-detection { ... } } } } }`.
//!
//! # See also
//!
//! - [bfd-liveness-detection (Protocols OSPF)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/bfd-liveness-detection-edit-protocols-ospf.html)

use crate::{arg_of, child_block, has_leaf};
use asterust::config::ConfigLine;

/// The BFD session of an OSPF interface.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OspfBfd {
    /// Milliseconds.
    pub minimum_interval: Option<String>,
    pub minimum_receive_interval: Option<String>,
    pub multiplier: Option<String>,
    /// `1` or `automatic`.
    pub version: Option<String>,
    pub holddown_interval: Option<String>,
    pub pdu_size: Option<String>,
    pub no_adaptation: bool,
    pub full_neighbors_only: bool,
    /// From `detection-time { threshold ...; }`.
    pub detection_time_threshold: Option<String>,
    /// From `transmit-interval { minimum-interval ...; }`.
    pub transmit_interval_minimum: Option<String>,
    /// From `transmit-interval { threshold ...; }`.
    pub transmit_interval_threshold: Option<String>,
    pub authentication_algorithm: Option<String>,
    /// The name of the key chain; the keys are not extracted.
    pub authentication_key_chain: Option<String>,
    pub authentication_loose_check: bool,
}

pub(super) fn parse_bfd(iface: &ConfigLine) -> Option<OspfBfd> {
    let bfd = child_block(iface, "bfd-liveness-detection")?;
    let field = |keyword: &str| arg_of(bfd, keyword).map(str::to_string);
    let nested = |block: &str, keyword: &str| {
        child_block(bfd, block)
            .and_then(|block| arg_of(block, keyword))
            .map(str::to_string)
    };
    Some(OspfBfd {
        minimum_interval: field("minimum-interval"),
        minimum_receive_interval: field("minimum-receive-interval"),
        multiplier: field("multiplier"),
        version: field("version"),
        holddown_interval: field("holddown-interval"),
        pdu_size: field("pdu-size"),
        no_adaptation: has_leaf(bfd, "no-adaptation"),
        full_neighbors_only: has_leaf(bfd, "full-neighbors-only"),
        detection_time_threshold: nested("detection-time", "threshold"),
        transmit_interval_minimum: nested("transmit-interval", "minimum-interval"),
        transmit_interval_threshold: nested("transmit-interval", "threshold"),
        authentication_algorithm: nested("authentication", "algorithm"),
        authentication_key_chain: nested("authentication", "key-chain"),
        authentication_loose_check: child_block(bfd, "authentication")
            .is_some_and(|auth| has_leaf(auth, "loose-check")),
    })
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
            interface ge-0/0/0.0 {
                bfd-liveness-detection {
                    minimum-interval 300;
                    multiplier 3;
                    version automatic;
                    no-adaptation;
                    full-neighbors-only;
                    detection-time {
                        threshold 900;
                    }
                    transmit-interval {
                        minimum-interval 100;
                        threshold 500;
                    }
                    authentication {
                        algorithm keyed-sha-1;
                        key-chain BFD-KEYS;
                        loose-check;
                    }
                }
            }
            interface ge-0/0/1.0 {
                bfd-liveness-detection {
                    minimum-receive-interval 50;
                }
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_session_and_its_nested_blocks() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let bfd = ospf.areas[0].interfaces[0].bfd_liveness_detection.as_ref().unwrap();
        assert_eq!(bfd.minimum_interval.as_deref(), Some("300"));
        assert_eq!(bfd.multiplier.as_deref(), Some("3"));
        assert_eq!(bfd.version.as_deref(), Some("automatic"));
        assert!(bfd.no_adaptation);
        assert!(bfd.full_neighbors_only);
        assert_eq!(bfd.detection_time_threshold.as_deref(), Some("900"));
        assert_eq!(bfd.transmit_interval_minimum.as_deref(), Some("100"));
        assert_eq!(bfd.transmit_interval_threshold.as_deref(), Some("500"));
        assert_eq!(bfd.authentication_algorithm.as_deref(), Some("keyed-sha-1"));
        assert_eq!(bfd.authentication_key_chain.as_deref(), Some("BFD-KEYS"));
        assert!(bfd.authentication_loose_check);
    }

    #[test]
    fn leaves_the_other_fields_unset() {
        let ospf = ospf(&juniper::parse_braces(CONFIG)).unwrap();
        let bfd = ospf.areas[0].interfaces[1].bfd_liveness_detection.as_ref().unwrap();
        assert_eq!(bfd.minimum_receive_interval.as_deref(), Some("50"));
        assert_eq!(bfd.minimum_interval, None);
        assert!(!bfd.no_adaptation);
        assert_eq!(bfd.authentication_algorithm, None);
    }
}
