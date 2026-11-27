//! `graceful-restart`
//! `protocols { bgp { group <name> { graceful-restart { ... } } } }`.
//!
//! # See also
//!
//! - [graceful-restart (Protocols BGP)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/graceful-restart-edit-protocols-bgp.html)
//! - [restart-time (BGP Graceful Restart)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/restart-time-edit-protocols-multiple.html)
//! - [stale-routes-time](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/stale-routes-time-edit-protocols-bgp.html)
//! - [disable (BGP Graceful Restart)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/disable-edit-protocols-bgp-graceful-restart.html)
//! - [long-lived (Graceful Restart for BGP Helper)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/long-lived-edit-protocols-bgp-graceful-restart.html)
//! - [receiver (Graceful Restart for BGP Helper)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/receiver-edit-protocols-bgp-graceful-restart.html)
//! - [forwarding-state-bit (Long-Lived Graceful Restart for BGP Restarter)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/forwarding-state-bit-edit-protocols-bgp-graceful-restart.html)
//! - [disable-notification-flag (BGP Graceful Restart)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/disable-notification-flag-edit-protocols-bgp-graceful-restart.html)
//! - [disable-notification-extensions (BGP Graceful Restart)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/disable-notification-extensions-edit-protocols-bgp-graceful-restart.html)
//! - [Understanding Graceful Restart for BGP](https://networkingdocs.hpe.com/documentation/us/en/software/junos/bgp/topics/topic-map/bgp-long-lived-graceful-restart.html)

use crate::{child_block, has_leaf, statement_words, value_after};
use asterust::config::ConfigLine;

/// The BGP `graceful-restart` settings. A bare `graceful-restart;` leaves every field unset. The per-family
/// settings are in [`BgpFamily`](crate::protocols::bgp::BgpFamily).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct BgpGracefulRestart {
    pub disable: bool,
    /// Seconds.
    pub restart_time: Option<String>,
    /// Seconds.
    pub stale_routes_time: Option<String>,
    /// `Some(true)` for `long-lived { receiver { enable; } }`, `Some(false)` for `disable`.
    pub long_lived_receiver: Option<bool>,
    /// `long-lived { advertise-to-non-llgr-neighbor { omit-no-export; } }`.
    pub omit_no_export: bool,
    /// `as-rr-client` or `from-fib`.
    pub forwarding_state_bit: Option<String>,
    pub disable_notification_flag: bool,
    pub disable_notification_extensions: bool,
}

pub(super) fn parse_graceful_restart(line: &ConfigLine) -> Option<BgpGracefulRestart> {
    let words = statement_words(line, "graceful-restart")?;
    let long_lived = child_block(line, "graceful-restart").and_then(|restart| child_block(restart, "long-lived"));
    let receiver = long_lived.and_then(|long_lived| child_block(long_lived, "receiver"));
    Some(BgpGracefulRestart {
        disable: words.contains(&"disable"),
        restart_time: value_after(&words, "restart-time"),
        stale_routes_time: value_after(&words, "stale-routes-time"),
        long_lived_receiver: receiver.and_then(|receiver| {
            if has_leaf(receiver, "enable") {
                Some(true)
            } else if has_leaf(receiver, "disable") {
                Some(false)
            } else {
                None
            }
        }),
        omit_no_export: long_lived
            .and_then(|long_lived| child_block(long_lived, "advertise-to-non-llgr-neighbor"))
            .is_some_and(|neighbor| has_leaf(neighbor, "omit-no-export")),
        forwarding_state_bit: value_after(&words, "forwarding-state-bit"),
        disable_notification_flag: words.contains(&"disable-notification-flag"),
        disable_notification_extensions: words.contains(&"disable-notification-extensions"),
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
        graceful-restart;
        group IBGP {
            graceful-restart {
                restart-time 180;
                stale-routes-time 400;
                long-lived {
                    receiver {
                        enable;
                    }
                    advertise-to-non-llgr-neighbor {
                        omit-no-export;
                    }
                }
            }
            neighbor 10.0.0.2 {
                graceful-restart {
                    disable;
                }
            }
        }
    }
}
"#;

    #[test]
    fn reads_timers_and_long_lived_helper_settings() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let global = bgp.options.graceful_restart.as_ref().unwrap();
        assert!(!global.disable);
        assert_eq!(global.restart_time, None);

        let group = bgp.groups[0].options.graceful_restart.as_ref().unwrap();
        assert_eq!(group.restart_time.as_deref(), Some("180"));
        assert_eq!(group.stale_routes_time.as_deref(), Some("400"));
        assert_eq!(group.long_lived_receiver, Some(true));
        assert!(group.omit_no_export);
    }

    #[test]
    fn reads_disable_at_neighbor_level() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let neighbor = bgp.groups[0].neighbors[0].options.graceful_restart.as_ref().unwrap();
        assert!(neighbor.disable);
        assert_eq!(neighbor.long_lived_receiver, None);
    }

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const NOTIFICATION: &str = r#"
protocols {
    bgp {
        graceful-restart {
            forwarding-state-bit as-rr-client;
            disable-notification-flag;
            disable-notification-extensions;
        }
    }
}
"#;

    #[test]
    fn reads_the_forwarding_state_bit_and_notification_flags() {
        let bgp = bgp(&juniper::parse_braces(NOTIFICATION)).unwrap();
        let restart = bgp.options.graceful_restart.unwrap();
        assert_eq!(restart.forwarding_state_bit.as_deref(), Some("as-rr-client"));
        assert!(restart.disable_notification_flag);
        assert!(restart.disable_notification_extensions);
    }
}
