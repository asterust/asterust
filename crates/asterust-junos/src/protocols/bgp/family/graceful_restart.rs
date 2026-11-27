//! `graceful-restart`
//! `protocols { bgp { group <name> { family <afi> { <safi> { graceful-restart { ... } } } } } }`.
//!
//! # See also
//!
//! - [graceful-restart (Long-Lived for BGP Restarter)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/graceful-restart-long-lived-per-family-edit-protocols-bgp.html)
//! - [long-lived (Graceful Restart for BGP Restarter)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/long-lived-per-family-edit-protocols-bgp-graceful-restart.html)
//! - [forwarding-state-bit (Per Family for BGP Graceful Restart)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/forwarding-state-bit-per-family-edit-protocols-bgp.html)
//! - [Understanding Graceful Restart for BGP](https://networkingdocs.hpe.com/documentation/us/en/software/junos/bgp/topics/topic-map/bgp-long-lived-graceful-restart.html)

use crate::{arg_of, child_block, has_leaf, statement_words, value_after};
use asterust::config::ConfigLine;

/// The `graceful-restart` statement of one address family, mainly the long-lived restarter settings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct FamilyGracefulRestart {
    /// `set` or `from-fib`.
    pub forwarding_state_bit: Option<String>,
    pub disable_notification_flag: bool,
    pub disable_notification_extensions: bool,
    /// `long-lived { restarter { disable; } }`.
    pub llgr_restarter_disable: bool,
    /// Seconds, or a duration such as `12h`, as written.
    pub llgr_restarter_stale_time: Option<String>,
}

pub(super) fn parse_family_graceful_restart(node: &ConfigLine) -> Option<FamilyGracefulRestart> {
    let words = statement_words(node, "graceful-restart")?;
    let restarter = child_block(node, "graceful-restart")
        .and_then(|restart| child_block(restart, "long-lived"))
        .and_then(|long_lived| child_block(long_lived, "restarter"));
    Some(FamilyGracefulRestart {
        forwarding_state_bit: value_after(&words, "forwarding-state-bit"),
        disable_notification_flag: words.contains(&"disable-notification-flag"),
        disable_notification_extensions: words.contains(&"disable-notification-extensions"),
        llgr_restarter_disable: restarter.is_some_and(|restarter| has_leaf(restarter, "disable")),
        llgr_restarter_stale_time: restarter
            .and_then(|restarter| arg_of(restarter, "stale-time"))
            .map(str::to_string),
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
        group VPN {
            family l2vpn {
                signaling {
                    graceful-restart {
                        forwarding-state-bit set;
                        long-lived {
                            restarter {
                                stale-time "2w1d 12:00:02";
                            }
                        }
                    }
                }
            }
            family inet {
                unicast {
                    graceful-restart {
                        disable-notification-flag;
                        long-lived {
                            restarter {
                                disable;
                            }
                        }
                    }
                }
                flow;
            }
        }
    }
}
"#;

    #[test]
    fn reads_the_forwarding_state_bit_and_stale_time() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let restart = bgp.groups[0].options.address_families[0]
            .graceful_restart
            .as_ref()
            .unwrap();
        assert_eq!(restart.forwarding_state_bit.as_deref(), Some("set"));
        assert_eq!(restart.llgr_restarter_stale_time.as_deref(), Some("2w1d 12:00:02"));
        assert!(!restart.llgr_restarter_disable);
    }

    #[test]
    fn reads_the_disabled_restarter_and_notification_flag() {
        let bgp = bgp(&juniper::parse_braces(CONFIG)).unwrap();
        let families = &bgp.groups[0].options.address_families;
        let restart = families[1].graceful_restart.as_ref().unwrap();
        assert!(restart.llgr_restarter_disable);
        assert!(restart.disable_notification_flag);
        assert!(!restart.disable_notification_extensions);
        assert_eq!(restart.llgr_restarter_stale_time, None);
        assert_eq!(families[2].graceful_restart, None);
    }
}
