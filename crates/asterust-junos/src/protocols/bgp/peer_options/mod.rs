//! `local-as`, `hold-time`, `authentication-algorithm` and the other peer options
//! `protocols { bgp { [group <name> { [neighbor <address> {] ... } } } }`; valid at the `bgp`, `group` and `neighbor` levels.
//!
//! # See also
//!
//! - [authentication-algorithm](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/authentication-algorithm-edit-protocols-bgp.html)
//! - [authentication-key (Protocols BGP and BMP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/authentication-key-edit-protocols-bgp.html)
//! - [authentication-key-chain (Protocols BGP and BMP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/authentication-key-chain-edit-protocols-bgp.html)
//! - [hold-time (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/hold-time-edit-protocols-bgp.html)
//! - [out-delay](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/out-delay-edit-protocols-bgp.html)
//! - [metric-out](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/metric-out-edit-protocols-bgp.html)
//! - [local-preference](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/local-preference-edit-protocols-bgp.html)
//! - [preference (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/preference-edit-protocols-bgp.html)
//! - [keep](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/keep-edit-protocols-bgp.html)
//! - [tcp-mss (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/tcp-mss-edit-protocols-bgp.html)
//! - [bgp-identifier](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/bgp-identifier-edit-protocols-bgp.html)
//! - [ipsec-sa (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/ipsec-sa-edit-protocols-bgp.html)
//! - [local-interface (IPv6)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/local-interface-edit-protocols-bgp.html)
//! - [advertise-peer-as](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/advertise-peer-as-edit-protocols-bgp.html)
//! - [no advertise-peer-as](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/no-advertise-peer-as-edit-protocols-bgp.html)
//! - [advertise-inactive](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/advertise-inactive-edit-protocols-bgp.html)
//! - [advertise-external](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/advertise-external-edit-protocols-bgp.html)
//! - [as-override](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/as-override-edit-protocols-bgp.html)
//! - [damping (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/damping-edit-protocols-bgp.html)
//! - [passive (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/passive-edit-protocols-bgp.html)
//! - [mtu-discovery](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/mtu-discovery-edit-protocols-bgp.html)
//! - [log-updown](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/log-updown-edit-protocols-bgp.html)
//! - [no-aggregator-id](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/no-aggregator-id-edit-protocols-bgp.html)
//! - [no-client-reflect](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/no-client-reflect-edit-protocols-bgp.html)
//! - [include-mp-next-hop](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/include-mp-next-hop-edit-protocols-bgp.html)
//! - [accept-remote-nexthop](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/accept-remote-nexthop-edit-protocols-bgp.html)
//! - [vpn-apply-export](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/vpn-apply-export-edit-protocols-bgp-vp.html)

mod local_as;
mod multihop;
mod remove_private;

pub use local_as::LocalAs;
pub use multihop::Multihop;
pub use remove_private::RemovePrivate;

use crate::{arg_of, has_leaf, statement_words, toggle};
use asterust::config::ConfigLine;
use local_as::parse_local_as;
use multihop::parse_multihop;
use remove_private::parse_remove_private;

/// The session settings shared by the `bgp`, `group` and `neighbor` levels. Settings inherited from an outer level
/// are not resolved. The `authentication-key` is not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpPeerOptions {
    pub local_as_options: Option<LocalAs>,
    pub multihop: Option<Multihop>,
    pub remove_private: Option<RemovePrivate>,
    /// `aes-128-cmac-96`, `hmac-sha-1-96` or `md5`.
    pub authentication_algorithm: Option<String>,
    /// An `authentication-key` is configured.
    pub has_authentication_key: bool,
    pub authentication_key_chain: Option<String>,
    /// Seconds.
    pub hold_time: Option<String>,
    /// Seconds.
    pub out_delay: Option<String>,
    pub tcp_mss: Option<String>,
    /// `all` or `none`.
    pub keep: Option<String>,
    pub local_preference: Option<String>,
    pub preference: Option<String>,
    pub bgp_identifier: Option<String>,
    pub ipsec_sa: Option<String>,
    /// The interface of an IPv6 link-local neighbor address.
    pub local_interface: Option<String>,
    /// The `metric-out` words as written, e.g. `"100"`, `"minimum-igp 10"` or `"igp delay-med-update"`.
    pub metric_out: Option<String>,
    /// `Some(true)` for `advertise-peer-as`, `Some(false)` for `no-advertise-peer-as`.
    pub advertise_peer_as: Option<bool>,
    pub advertise_external: bool,
    /// `advertise-external conditional`.
    pub advertise_external_conditional: bool,
    pub passive: bool,
    pub mtu_discovery: bool,
    pub advertise_inactive: bool,
    pub as_override: bool,
    pub damping: bool,
    pub no_aggregator_id: bool,
    pub no_client_reflect: bool,
    pub log_updown: bool,
    pub include_mp_next_hop: bool,
    pub accept_remote_nexthop: bool,
    pub vpn_apply_export: bool,
}

/// Reads the peer options of a `bgp`, `group` or `neighbor` block.
pub(super) fn parse_peer_options(line: &ConfigLine) -> BgpPeerOptions {
    let field = |keyword: &str| arg_of(line, keyword).map(str::to_string);
    let external = statement_words(line, "advertise-external");
    BgpPeerOptions {
        local_as_options: parse_local_as(line),
        multihop: parse_multihop(line),
        remove_private: parse_remove_private(line),
        authentication_algorithm: field("authentication-algorithm"),
        has_authentication_key: arg_of(line, "authentication-key").is_some(),
        authentication_key_chain: field("authentication-key-chain"),
        hold_time: field("hold-time"),
        out_delay: field("out-delay"),
        tcp_mss: field("tcp-mss"),
        keep: field("keep"),
        local_preference: field("local-preference"),
        preference: field("preference"),
        bgp_identifier: field("bgp-identifier"),
        ipsec_sa: field("ipsec-sa"),
        local_interface: field("local-interface"),
        metric_out: statement_words(line, "metric-out").map(|words| words.join(" ")),
        advertise_peer_as: toggle(line, "advertise-peer-as"),
        advertise_external: external.is_some(),
        advertise_external_conditional: external.is_some_and(|words| words.contains(&"conditional")),
        passive: has_leaf(line, "passive"),
        mtu_discovery: has_leaf(line, "mtu-discovery"),
        advertise_inactive: has_leaf(line, "advertise-inactive"),
        as_override: has_leaf(line, "as-override"),
        damping: has_leaf(line, "damping"),
        no_aggregator_id: has_leaf(line, "no-aggregator-id"),
        no_client_reflect: has_leaf(line, "no-client-reflect"),
        log_updown: has_leaf(line, "log-updown"),
        include_mp_next_hop: has_leaf(line, "include-mp-next-hop"),
        accept_remote_nexthop: has_leaf(line, "accept-remote-nexthop"),
        vpn_apply_export: has_leaf(line, "vpn-apply-export"),
    }
}

#[cfg(test)]
mod tests {
    use crate::protocols::bgp::bgp;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    bgp {
        hold-time 30;
        precision-timers;
        group EBGP {
            type external;
            peer-as 65001;
            local-as 65000 loops 2 private no-prepend-global-as;
            multihop {
                ttl 5;
                no-nexthop-change;
            }
            authentication-algorithm hmac-sha-1-96;
            authentication-key "$9$secret"; ## SECRET-DATA
            hold-time 90;
            out-delay 10;
            tcp-mss 1400;
            keep all;
            local-preference 200;
            preference 100;
            metric-out minimum-igp 10;
            remove-private all replace;
            advertise-peer-as;
            advertise-external conditional;
            passive;
            mtu-discovery;
            as-override;
            damping;
            log-updown;
            include-mp-next-hop;
            neighbor 192.0.2.2 {
                multihop ttl 3;
                authentication-key-chain BGP-KEYS;
                no-advertise-peer-as;
                local-interface ge-0/0/0.0;
                remove-private;
            }
        }
    }
}
"#;

    #[test]
    fn extracts_group_session_options_without_the_key() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let options = &bgp.groups[0].options;

        let local_as = options.local_as_options.as_ref().unwrap();
        assert_eq!(local_as.autonomous_system.as_deref(), Some("65000"));
        assert_eq!(local_as.loops.as_deref(), Some("2"));
        assert!(local_as.private);
        assert!(!local_as.alias);
        assert!(local_as.no_prepend_global_as);

        let multihop = options.multihop.as_ref().unwrap();
        assert_eq!(multihop.ttl.as_deref(), Some("5"));
        assert!(multihop.no_nexthop_change);

        assert_eq!(options.authentication_algorithm.as_deref(), Some("hmac-sha-1-96"));
        assert!(options.has_authentication_key);
        assert_eq!(options.hold_time.as_deref(), Some("90"));
        assert_eq!(options.out_delay.as_deref(), Some("10"));
        assert_eq!(options.tcp_mss.as_deref(), Some("1400"));
        assert_eq!(options.keep.as_deref(), Some("all"));
        assert_eq!(options.local_preference.as_deref(), Some("200"));
        assert_eq!(options.preference.as_deref(), Some("100"));
        assert_eq!(options.metric_out.as_deref(), Some("minimum-igp 10"));
        assert_eq!(options.advertise_peer_as, Some(true));
        assert!(options.advertise_external);
        assert!(options.advertise_external_conditional);
        assert!(options.passive);
        assert!(options.mtu_discovery);
        assert!(options.as_override);
        assert!(options.damping);
        assert!(options.log_updown);
        assert!(options.include_mp_next_hop);
        assert!(!options.no_aggregator_id);

        let remove_private = options.remove_private.as_ref().unwrap();
        assert!(remove_private.all);
        assert!(remove_private.replace);
        assert!(!remove_private.nearest);
    }

    #[test]
    fn extracts_neighbor_options_and_older_multihop_form() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let options = &bgp.groups[0].neighbors[0].options;
        assert_eq!(options.multihop.as_ref().unwrap().ttl.as_deref(), Some("3"));
        assert_eq!(options.authentication_key_chain.as_deref(), Some("BGP-KEYS"));
        assert!(!options.has_authentication_key);
        assert_eq!(options.advertise_peer_as, Some(false));
        assert_eq!(options.local_interface.as_deref(), Some("ge-0/0/0.0"));
        assert!(!options.remove_private.as_ref().unwrap().all);
        assert_eq!(options.hold_time, None);
    }

    #[test]
    fn extracts_global_options_and_flags() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        assert_eq!(bgp.options.hold_time.as_deref(), Some("30"));
        assert!(bgp.precision_timers);
        assert!(!bgp.disable);
    }

    #[test]
    fn peer_options_default_to_unset() {
        let bgp = bgp(&juniper::parse_braces(
            "protocols {\n    bgp {\n        group G {\n            type internal;\n        }\n    }\n}\n",
        ))
        .unwrap();
        let options = &bgp.groups[0].options;
        assert!(options.local_as_options.is_none());
        assert!(options.multihop.is_none());
        assert!(options.remove_private.is_none());
        assert_eq!(options.advertise_peer_as, None);
        assert!(!options.advertise_external);
        assert!(!options.passive);
    }
}
