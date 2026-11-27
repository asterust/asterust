//! `bfd-liveness-detection`
//! `protocols { isis { interface <name> { bfd-liveness-detection { ... } } } }`.
//!
//! # See also
//!
//! - [bfd-liveness-detection (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/bfd-liveness-detection-edit-protocols-isis.html)

use crate::{arg_of, child_block, has_leaf};
use asterust::config::ConfigLine;

/// The BFD session of an IS-IS interface.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct IsisBfd {
    /// Milliseconds.
    pub minimum_interval: Option<String>,
    pub minimum_receive_interval: Option<String>,
    pub multiplier: Option<String>,
    /// `1` or `automatic`.
    pub version: Option<String>,
    pub pdu_size: Option<String>,
    pub no_adaptation: bool,
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

pub(super) fn parse_bfd(iface: &ConfigLine) -> Option<IsisBfd> {
    let bfd = child_block(iface, "bfd-liveness-detection")?;
    let field = |keyword: &str| arg_of(bfd, keyword).map(str::to_string);
    let nested = |block: &str, keyword: &str| {
        child_block(bfd, block)
            .and_then(|block| arg_of(block, keyword))
            .map(str::to_string)
    };
    Some(IsisBfd {
        minimum_interval: field("minimum-interval"),
        minimum_receive_interval: field("minimum-receive-interval"),
        multiplier: field("multiplier"),
        version: field("version"),
        pdu_size: field("pdu-size"),
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
    use crate::protocols::isis::isis;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    isis {
        interface ge-1/2/0.0 {
            bfd-liveness-detection {
                minimum-interval 100;
                authentication {
                    key-chain secret123;
                    algorithm meticulous-keyed-md5;
                }
            }
        }
        interface ge-1/2/1.0 {
            bfd-liveness-detection {
                minimum-receive-interval 50;
                multiplier 5;
                version 1;
                no-adaptation;
                detection-time {
                    threshold 900;
                }
                transmit-interval {
                    minimum-interval 100;
                    threshold 500;
                }
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_authentication_block() {
        let isis = isis(&juniper::parse_braces(CONFIG)).unwrap();
        let bfd = isis.interfaces[0].bfd_liveness_detection.as_ref().unwrap();
        assert_eq!(bfd.minimum_interval.as_deref(), Some("100"));
        assert_eq!(bfd.authentication_key_chain.as_deref(), Some("secret123"));
        assert_eq!(bfd.authentication_algorithm.as_deref(), Some("meticulous-keyed-md5"));
        assert!(!bfd.authentication_loose_check);
    }

    #[test]
    fn reads_the_other_fields() {
        let isis = isis(&juniper::parse_braces(CONFIG)).unwrap();
        let bfd = isis.interfaces[1].bfd_liveness_detection.as_ref().unwrap();
        assert_eq!(bfd.minimum_receive_interval.as_deref(), Some("50"));
        assert_eq!(bfd.multiplier.as_deref(), Some("5"));
        assert_eq!(bfd.version.as_deref(), Some("1"));
        assert!(bfd.no_adaptation);
        assert_eq!(bfd.detection_time_threshold.as_deref(), Some("900"));
        assert_eq!(bfd.transmit_interval_minimum.as_deref(), Some("100"));
        assert_eq!(bfd.transmit_interval_threshold.as_deref(), Some("500"));
        assert_eq!(bfd.minimum_interval, None);
    }
}
