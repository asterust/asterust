//! `level`
//! `protocols { isis { interface <name> { level <1|2> { ... } } } }`.
//!
//! # See also
//!
//! - [level (IS-IS Interfaces)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/level-edit-protocols-isis-interface.html)
//! - [metric (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/metric-edit-protocols-isis.html)
//! - [hold-time (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/hold-time-edit-protocols-isis.html)
//! - [hello-interval (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/hello-interval-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [priority (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/priority-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [te-metric (Protocols IS-IS)](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/te-metric-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [hello-authentication-key](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/hello-authentication-key-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [hello-authentication-type](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/hello-authentication-type-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [ipv4-multicast-metric](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/ipv4-multicast-metric-edit-protocols-isis.html), from the Junos 12.3 archive
//! - [ipv6-unicast-metric](https://networkingdocs.hpe.com/documentation/en_US/junos12.3/topics/reference/configuration-statement/ipv6-unicast-metric-edit-protocols-isis.html), from the Junos 12.3 archive

use crate::protocols::isis::srv6::{parse_adjacency_segments, parse_lan_neighbor_segments, IsisSrv6AdjacencySegment};
use crate::{active_children, split_name, value_after};
use asterust::config::ConfigLine;

/// A `level <1|2>` entry of an IS-IS interface, written `level 1 disable;`, `level 2 metric 10;` or as a block.
/// The `hello-authentication-key` is never extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct IsisInterfaceLevel {
    /// `1` or `2`.
    pub level: String,
    pub disable: bool,
    pub passive: bool,
    pub metric: Option<String>,
    /// Seconds.
    pub hello_interval: Option<String>,
    /// Seconds.
    pub hold_time: Option<String>,
    pub priority: Option<String>,
    pub te_metric: Option<String>,
    pub ipv4_multicast_metric: Option<String>,
    pub ipv6_multicast_metric: Option<String>,
    pub ipv6_unicast_metric: Option<String>,
    /// `md5` or `simple`.
    pub hello_authentication_type: Option<String>,
    /// A `hello-authentication-key` is configured.
    pub has_hello_authentication_key: bool,
    pub hello_authentication_key_chain: Option<String>,
    /// The `srv6-adjacency-segment` locators of the level and of its `lan-neighbor` entries.
    pub srv6_adjacency_segments: Vec<IsisSrv6AdjacencySegment>,
}

/// `arg` is the text after `level`, e.g. `1 disable` or `2`.
pub(super) fn parse_level(arg: &str, level: &ConfigLine) -> IsisInterfaceLevel {
    let (number, rest) = split_name(arg);
    let words: Vec<&str> = rest
        .split_whitespace()
        .chain(active_children(level).flat_map(|child| child.text.split_whitespace()))
        .collect();
    IsisInterfaceLevel {
        level: number.to_string(),
        disable: words.contains(&"disable"),
        passive: words.contains(&"passive"),
        metric: value_after(&words, "metric"),
        hello_interval: value_after(&words, "hello-interval"),
        hold_time: value_after(&words, "hold-time"),
        priority: value_after(&words, "priority"),
        te_metric: value_after(&words, "te-metric"),
        ipv4_multicast_metric: value_after(&words, "ipv4-multicast-metric"),
        ipv6_multicast_metric: value_after(&words, "ipv6-multicast-metric"),
        ipv6_unicast_metric: value_after(&words, "ipv6-unicast-metric"),
        hello_authentication_type: value_after(&words, "hello-authentication-type"),
        has_hello_authentication_key: words.contains(&"hello-authentication-key"),
        hello_authentication_key_chain: value_after(&words, "hello-authentication-key-chain"),
        srv6_adjacency_segments: {
            let mut segments = parse_adjacency_segments(level, None);
            segments.extend(parse_lan_neighbor_segments(level));
            segments
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::protocols::isis::isis;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
protocols {
    isis {
        interface fe-1/2/0.0 {
            level 1 disable;
        }
        interface ge-0/0/0.0 {
            level 1 {
                metric 20;
                hello-interval 5;
                hold-time 15;
                priority 100;
                hello-authentication-key "$9$secret"; ## SECRET-DATA
                hello-authentication-type md5;
            }
            level 2 metric 10;
        }
        interface ge-0/0/1.0 {
            level 2 {
                te-metric 30;
                ipv4-multicast-metric 7;
                ipv6-unicast-metric 40;
                hello-authentication-key-chain ISIS-KEYS;
                passive;
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_one_line_forms() {
        let isis = isis(&juniper::parse_braces(CONFIG)).unwrap();
        let disabled = &isis.interfaces[0].levels[0];
        assert_eq!(disabled.level, "1");
        assert!(disabled.disable);

        let metric = &isis.interfaces[1].levels[1];
        assert_eq!(metric.level, "2");
        assert_eq!(metric.metric.as_deref(), Some("10"));
        assert!(!metric.disable);
    }

    #[test]
    fn reads_the_block_form_without_the_key() {
        let isis = isis(&juniper::parse_braces(CONFIG)).unwrap();
        let level = &isis.interfaces[1].levels[0];
        assert_eq!(level.metric.as_deref(), Some("20"));
        assert_eq!(level.hello_interval.as_deref(), Some("5"));
        assert_eq!(level.hold_time.as_deref(), Some("15"));
        assert_eq!(level.priority.as_deref(), Some("100"));
        assert_eq!(level.hello_authentication_type.as_deref(), Some("md5"));
        assert!(level.has_hello_authentication_key);
        assert_eq!(level.hello_authentication_key_chain, None);
    }

    #[test]
    fn reads_the_remaining_metrics() {
        let isis = isis(&juniper::parse_braces(CONFIG)).unwrap();
        let level = &isis.interfaces[2].levels[0];
        assert_eq!(level.te_metric.as_deref(), Some("30"));
        assert_eq!(level.ipv4_multicast_metric.as_deref(), Some("7"));
        assert_eq!(level.ipv6_unicast_metric.as_deref(), Some("40"));
        assert_eq!(level.ipv6_multicast_metric, None);
        assert_eq!(level.hello_authentication_key_chain.as_deref(), Some("ISIS-KEYS"));
        assert!(level.passive);
        assert!(!level.has_hello_authentication_key);
    }
}
