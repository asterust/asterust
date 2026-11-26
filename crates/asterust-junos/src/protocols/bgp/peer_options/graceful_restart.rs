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

use crate::{child_block, has_leaf, statement_words, value_after};
use asterust::config::ConfigLine;

/// The BGP `graceful-restart` settings. A bare `graceful-restart;` leaves every field unset. The per-family
/// restarter settings are not extracted.
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
}
