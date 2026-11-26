//! `bfd-liveness-detection`
//! `protocols { bgp { group <name> { bfd-liveness-detection { ... } } } }`.
//!
//! # See also
//!
//! - [bfd-liveness-detection (BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/bfd-liveness-detection-edit-protocols-bgp.html)
//! - [authentication (BGP BFD Liveness Detection)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/authentication-edit-protocols-bgp-liveness-detection.html)
//! - [detection-time (BFD Liveness Detection)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/detection-time-edit-protocols-bgp-bfd-liveness-detection.html)
//! - [transmit-interval (BFD Liveness Detection)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/transmit-interval-edit-protocols-bgp-liveness-detection.html)

use crate::{arg_of, child_block, has_leaf};
use asterust::config::ConfigLine;

/// The BFD session of a BGP peer. The `strict-bfd` block is not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpBfd {
    /// Milliseconds.
    pub minimum_interval: Option<String>,
    pub minimum_receive_interval: Option<String>,
    pub multiplier: Option<String>,
    /// `1` or `automatic`.
    pub version: Option<String>,
    /// `automatic`, `multihop` or `single-hop`.
    pub session_mode: Option<String>,
    pub pdu_size: Option<String>,
    pub holddown_interval: Option<String>,
    pub no_adaptation: bool,
    /// From `detection-time { threshold ...; }`.
    pub detection_time_threshold: Option<String>,
    /// From `transmit-interval { minimum-interval ...; }`.
    pub transmit_interval_minimum: Option<String>,
    /// From `transmit-interval { threshold ...; }`.
    pub transmit_interval_threshold: Option<String>,
    /// From `authentication { algorithm ...; }`, e.g. `keyed-sha-1`.
    pub authentication_algorithm: Option<String>,
    /// The name of the key chain; the keys are not extracted.
    pub authentication_key_chain: Option<String>,
    pub authentication_loose_check: bool,
}

pub(super) fn parse_bfd(line: &ConfigLine) -> Option<BgpBfd> {
    let bfd = child_block(line, "bfd-liveness-detection")?;
    let field = |keyword: &str| arg_of(bfd, keyword).map(str::to_string);
    let nested = |block: &str, keyword: &str| {
        child_block(bfd, block)
            .and_then(|block| arg_of(block, keyword))
            .map(str::to_string)
    };
    Some(BgpBfd {
        minimum_interval: field("minimum-interval"),
        minimum_receive_interval: field("minimum-receive-interval"),
        multiplier: field("multiplier"),
        version: field("version"),
        session_mode: field("session-mode"),
        pdu_size: field("pdu-size"),
        holddown_interval: field("holddown-interval"),
        no_adaptation: has_leaf(bfd, "no-adaptation"),
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
    use crate::protocols::bgp::bgp;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    bgp {
        group EBGP {
            bfd-liveness-detection {
                minimum-interval 300;
                multiplier 3;
                session-mode single-hop;
                no-adaptation;
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
            neighbor 192.0.2.2 {
                bfd-liveness-detection {
                    minimum-interval 50;
                }
            }
            neighbor 192.0.2.3;
        }
    }
}
"#;

    #[test]
    fn reads_the_session_and_its_nested_blocks() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let group = bgp.groups[0].options.bfd_liveness_detection.as_ref().unwrap();
        assert_eq!(group.minimum_interval.as_deref(), Some("300"));
        assert_eq!(group.multiplier.as_deref(), Some("3"));
        assert_eq!(group.session_mode.as_deref(), Some("single-hop"));
        assert!(group.no_adaptation);
        assert_eq!(group.detection_time_threshold.as_deref(), Some("900"));
        assert_eq!(group.transmit_interval_minimum.as_deref(), Some("100"));
        assert_eq!(group.transmit_interval_threshold.as_deref(), Some("500"));
        assert_eq!(group.authentication_algorithm.as_deref(), Some("keyed-sha-1"));
        assert_eq!(group.authentication_key_chain.as_deref(), Some("BFD-KEYS"));
        assert!(group.authentication_loose_check);
    }

    #[test]
    fn reads_the_neighbor_level_and_leaves_others_unset() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let neighbors = &bgp.groups[0].neighbors;
        let bfd = neighbors[0].options.bfd_liveness_detection.as_ref().unwrap();
        assert_eq!(bfd.minimum_interval.as_deref(), Some("50"));
        assert!(!bfd.no_adaptation);
        assert_eq!(bfd.authentication_algorithm, None);
        assert_eq!(neighbors[1].options.bfd_liveness_detection, None);
    }
}
