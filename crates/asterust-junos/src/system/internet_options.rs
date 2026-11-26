//! `system { internet-options { ... } }`.

use crate::{arg_of, has_leaf, statement_words, toggle, value_after};
use asterust::config::ConfigLine;

/// An `icmpv4-rate-limit` or `icmpv6-rate-limit`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct IcmpRateLimit {
    /// Seconds.
    pub bucket_size: Option<String>,
    /// Packets per second.
    pub packet_rate: Option<String>,
}

/// The `system internet-options` configuration.
///
/// Options that come as an enable/disable pair (`path-mtu-discovery` and `no-path-mtu-discovery`) are `None` when
/// neither is written, `Some(true)` for the plain form and `Some(false)` for the `no-` form.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct InternetOptions {
    pub path_mtu_discovery: Option<bool>,
    pub ipv6_path_mtu_discovery: Option<bool>,
    pub gre_path_mtu_discovery: Option<bool>,
    pub ipip_path_mtu_discovery: Option<bool>,
    pub ipv6_reject_zero_hop_limit: Option<bool>,
    pub source_quench: Option<bool>,
    pub icmpv4_rate_limit: Option<IcmpRateLimit>,
    pub icmpv6_rate_limit: Option<IcmpRateLimit>,
    pub tcp_drop_synfin_set: bool,
    pub no_tcp_rfc1323: bool,
    pub no_tcp_rfc1323_paws: bool,
    /// `drop-all-tcp` or `drop-tcp-with-syn-only`.
    pub no_tcp_reset: Option<String>,
    pub tcp_mss: Option<String>,
    pub source_port_upper_limit: Option<String>,
    pub ipv6_duplicate_addr_detection_transmits: Option<String>,
}

pub(super) fn parse_internet_options(options: &ConfigLine) -> InternetOptions {
    let field = |keyword: &str| arg_of(options, keyword).map(str::to_string);
    InternetOptions {
        path_mtu_discovery: toggle(options, "path-mtu-discovery"),
        ipv6_path_mtu_discovery: toggle(options, "ipv6-path-mtu-discovery"),
        gre_path_mtu_discovery: toggle(options, "gre-path-mtu-discovery"),
        ipip_path_mtu_discovery: toggle(options, "ipip-path-mtu-discovery"),
        ipv6_reject_zero_hop_limit: toggle(options, "ipv6-reject-zero-hop-limit"),
        source_quench: toggle(options, "source-quench"),
        icmpv4_rate_limit: rate_limit(options, "icmpv4-rate-limit"),
        icmpv6_rate_limit: rate_limit(options, "icmpv6-rate-limit"),
        tcp_drop_synfin_set: has_leaf(options, "tcp-drop-synfin-set"),
        no_tcp_rfc1323: has_leaf(options, "no-tcp-rfc1323"),
        no_tcp_rfc1323_paws: has_leaf(options, "no-tcp-rfc1323-paws"),
        no_tcp_reset: field("no-tcp-reset"),
        tcp_mss: field("tcp-mss"),
        source_port_upper_limit: statement_words(options, "source-port")
            .and_then(|words| value_after(&words, "upper-limit")),
        ipv6_duplicate_addr_detection_transmits: field("ipv6-duplicate-addr-detection-transmits"),
    }
}

/// `<keyword> bucket-size <n> packet-rate <n>`, on one line or as one child statement per option.
fn rate_limit(options: &ConfigLine, keyword: &str) -> Option<IcmpRateLimit> {
    let words = statement_words(options, keyword)?;
    Some(IcmpRateLimit {
        bucket_size: value_after(&words, "bucket-size"),
        packet_rate: value_after(&words, "packet-rate"),
    })
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax page. See the README "Sources".
    const INTERNET_OPTIONS: &str = r#"
system {
    internet-options {
        icmpv4-rate-limit bucket-size 10 packet-rate 500;
        icmpv6-rate-limit {
            bucket-size 3;
            packet-rate 200;
        }
        no-ipv6-reject-zero-hop-limit;
        no-source-quench;
        path-mtu-discovery;
        no-ipv6-path-mtu-discovery;
        no-tcp-reset drop-all-tcp;
        no-tcp-rfc1323;
        no-tcp-rfc1323-paws;
        tcp-drop-synfin-set;
        tcp-mss 1400;
        ipv6-duplicate-addr-detection-transmits 5;
        source-port {
            upper-limit 40000;
        }
    }
}
"#;

    #[test]
    fn extracts_toggle_options() {
        let options = system(&juniper::parse_braces(INTERNET_OPTIONS))
            .unwrap()
            .internet_options
            .unwrap();
        assert_eq!(options.path_mtu_discovery, Some(true));
        assert_eq!(options.ipv6_path_mtu_discovery, Some(false));
        assert_eq!(options.ipv6_reject_zero_hop_limit, Some(false));
        assert_eq!(options.source_quench, Some(false));
        assert_eq!(options.gre_path_mtu_discovery, None);
        assert_eq!(options.ipip_path_mtu_discovery, None);
    }

    #[test]
    fn extracts_icmp_rate_limits_in_line_and_block_form() {
        let options = system(&juniper::parse_braces(INTERNET_OPTIONS))
            .unwrap()
            .internet_options
            .unwrap();
        let v4 = options.icmpv4_rate_limit.unwrap();
        assert_eq!(v4.bucket_size.as_deref(), Some("10"));
        assert_eq!(v4.packet_rate.as_deref(), Some("500"));
        let v6 = options.icmpv6_rate_limit.unwrap();
        assert_eq!(v6.bucket_size.as_deref(), Some("3"));
        assert_eq!(v6.packet_rate.as_deref(), Some("200"));
    }

    #[test]
    fn extracts_tcp_and_port_options() {
        let options = system(&juniper::parse_braces(INTERNET_OPTIONS))
            .unwrap()
            .internet_options
            .unwrap();
        assert!(options.tcp_drop_synfin_set);
        assert!(options.no_tcp_rfc1323);
        assert!(options.no_tcp_rfc1323_paws);
        assert_eq!(options.no_tcp_reset.as_deref(), Some("drop-all-tcp"));
        assert_eq!(options.tcp_mss.as_deref(), Some("1400"));
        assert_eq!(options.source_port_upper_limit.as_deref(), Some("40000"));
        assert_eq!(options.ipv6_duplicate_addr_detection_transmits.as_deref(), Some("5"));
    }

    #[test]
    fn unset_options_are_none_or_false() {
        let options = system(&juniper::parse_braces(
            "system {\n    internet-options {\n        tcp-mss 1300;\n    }\n}\n",
        ))
        .unwrap()
        .internet_options
        .unwrap();
        assert_eq!(options.path_mtu_discovery, None);
        assert_eq!(options.icmpv4_rate_limit, None);
        assert!(!options.tcp_drop_synfin_set);
        assert_eq!(options.source_port_upper_limit, None);
    }

    #[test]
    fn no_internet_options_is_none() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.internet_options.is_none());
    }
}
