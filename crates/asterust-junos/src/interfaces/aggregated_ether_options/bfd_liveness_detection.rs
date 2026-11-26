//! Extracts the `bfd-liveness-detection` session of an aggregated Ethernet bundle.
//!
//! Configuration path: `interfaces { ae<n> { aggregated-ether-options { bfd-liveness-detection { ... } } } }`.
//!
//! # See also
//!
//! - [bfd-liveness-detection (LAG)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/bfd-liveness-detection-edit-interfaces-aex-aggregated-ether-options.html)

use crate::{arg_of, child_block, has_leaf};
use asterust::config::ConfigLine;

/// The BFD session of an aggregated Ethernet bundle. The `damping` block is not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BfdLivenessDetection {
    pub local_address: Option<String>,
    pub neighbor: Option<String>,
    /// Milliseconds.
    pub minimum_interval: Option<String>,
    pub minimum_receive_interval: Option<String>,
    pub multiplier: Option<String>,
    /// `1` or `automatic`.
    pub version: Option<String>,
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
    /// The name of the `security authentication-key-chains` entry; the keys are not extracted.
    pub authentication_key_chain: Option<String>,
    pub authentication_loose_check: bool,
}

pub(super) fn parse_bfd_liveness_detection(bfd: &ConfigLine) -> BfdLivenessDetection {
    let field = |keyword: &str| arg_of(bfd, keyword).map(str::to_string);
    let nested = |block: &str, keyword: &str| {
        child_block(bfd, block)
            .and_then(|block| arg_of(block, keyword))
            .map(str::to_string)
    };
    let authentication = child_block(bfd, "authentication");
    BfdLivenessDetection {
        local_address: field("local-address"),
        neighbor: field("neighbor"),
        minimum_interval: field("minimum-interval"),
        minimum_receive_interval: field("minimum-receive-interval"),
        multiplier: field("multiplier"),
        version: field("version"),
        holddown_interval: field("holddown-interval"),
        no_adaptation: has_leaf(bfd, "no-adaptation"),
        detection_time_threshold: nested("detection-time", "threshold"),
        transmit_interval_minimum: nested("transmit-interval", "minimum-interval"),
        transmit_interval_threshold: nested("transmit-interval", "threshold"),
        authentication_algorithm: nested("authentication", "algorithm"),
        authentication_key_chain: nested("authentication", "key-chain"),
        authentication_loose_check: authentication.is_some_and(|auth| has_leaf(auth, "loose-check")),
    }
}

#[cfg(test)]
mod tests {
    use crate::interfaces::interfaces;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    #[test]
    fn extracts_bfd_liveness_detection() {
        let ifaces = interfaces(&juniper::parse_braces(
            r#"
interfaces {
    ae0 {
        aggregated-ether-options {
            bfd-liveness-detection {
                local-address 192.0.2.1;
                neighbor 192.0.2.2;
                minimum-interval 300;
                minimum-receive-interval 400;
                multiplier 3;
                version automatic;
                holddown-interval 1000;
                no-adaptation;
                detection-time {
                    threshold 5000;
                }
                transmit-interval {
                    minimum-interval 250;
                    threshold 2000;
                }
                authentication {
                    algorithm keyed-sha-1;
                    key-chain BFD-KEYS;
                    loose-check;
                }
            }
        }
    }
}
"#,
        ));
        let bfd = ifaces[0]
            .aggregated_ether_options
            .as_ref()
            .unwrap()
            .bfd_liveness_detection
            .as_ref()
            .unwrap();
        assert_eq!(bfd.local_address.as_deref(), Some("192.0.2.1"));
        assert_eq!(bfd.neighbor.as_deref(), Some("192.0.2.2"));
        assert_eq!(bfd.minimum_interval.as_deref(), Some("300"));
        assert_eq!(bfd.minimum_receive_interval.as_deref(), Some("400"));
        assert_eq!(bfd.multiplier.as_deref(), Some("3"));
        assert_eq!(bfd.version.as_deref(), Some("automatic"));
        assert_eq!(bfd.holddown_interval.as_deref(), Some("1000"));
        assert!(bfd.no_adaptation);
        assert_eq!(bfd.detection_time_threshold.as_deref(), Some("5000"));
        assert_eq!(bfd.transmit_interval_minimum.as_deref(), Some("250"));
        assert_eq!(bfd.transmit_interval_threshold.as_deref(), Some("2000"));
        assert_eq!(bfd.authentication_algorithm.as_deref(), Some("keyed-sha-1"));
        assert_eq!(bfd.authentication_key_chain.as_deref(), Some("BFD-KEYS"));
        assert!(bfd.authentication_loose_check);
    }

    #[test]
    fn bare_bfd_liveness_detection_has_no_settings() {
        let ifaces = interfaces(&juniper::parse_braces(
            "interfaces {\n    ae0 {\n        aggregated-ether-options {\n            bfd-liveness-detection {\n                neighbor 192.0.2.2;\n            }\n        }\n    }\n}\n",
        ));
        let bfd = ifaces[0]
            .aggregated_ether_options
            .as_ref()
            .unwrap()
            .bfd_liveness_detection
            .as_ref()
            .unwrap();
        assert_eq!(bfd.neighbor.as_deref(), Some("192.0.2.2"));
        assert_eq!(bfd.local_address, None);
        assert!(!bfd.no_adaptation);
        assert_eq!(bfd.authentication_algorithm, None);
        assert!(!bfd.authentication_loose_check);
    }
}
