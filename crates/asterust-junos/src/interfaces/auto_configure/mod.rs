//! Extracts the `auto-configure` settings that create VLAN interfaces dynamically.
//!
//! Configuration path: `interfaces { <name> { auto-configure { ... } } }`.
//!
//! # See also
//!
//! - [auto-configure](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/auto-configure-edit-interfaces.html)

mod ranges;

pub use ranges::{AutoConfigureProfile, AutoConfigureRanges};

use crate::{child_block, has_leaf};
use asterust::config::ConfigLine;
use ranges::parse_ranges;

/// The `auto-configure` settings that create VLAN interfaces dynamically.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AutoConfigure {
    pub vlan_ranges: Option<AutoConfigureRanges>,
    pub stacked_vlan_ranges: Option<AutoConfigureRanges>,
    pub remove_when_no_subscribers: bool,
}

pub(super) fn parse_auto_configure(auto_configure: &ConfigLine) -> AutoConfigure {
    AutoConfigure {
        vlan_ranges: child_block(auto_configure, "vlan-ranges").map(parse_ranges),
        stacked_vlan_ranges: child_block(auto_configure, "stacked-vlan-ranges").map(parse_ranges),
        remove_when_no_subscribers: has_leaf(auto_configure, "remove-when-no-subscribers"),
    }
}

#[cfg(test)]
mod tests {
    use crate::interfaces::interfaces;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax pages. See the README "Sources".
    const CONFIG: &str = r#"
interfaces {
    ge-1/0/0 {
        auto-configure {
            remove-when-no-subscribers;
            vlan-ranges {
                access-profile AP;
                authentication {
                    password "$9$secret"; ## SECRET-DATA
                    username-include {
                        mac-address;
                    }
                }
                dynamic-profile DP1 {
                    accept dhcp-v4;
                    accept pppoe;
                    access-profile DAP;
                    ranges 100-200;
                }
                dynamic-profile DP2 {
                    ranges any-any;
                }
            }
            stacked-vlan-ranges {
                dynamic-profile SDP {
                    accept any;
                    ranges 100-200,any;
                }
            }
        }
    }
}
"#;

    #[test]
    fn extracts_vlan_ranges_without_the_password() {
        let ifaces = interfaces(&juniper::parse_braces(CONFIG));
        let auto = ifaces[0].auto_configure.as_ref().unwrap();
        assert!(auto.remove_when_no_subscribers);

        let ranges = auto.vlan_ranges.as_ref().unwrap();
        assert_eq!(ranges.access_profile.as_deref(), Some("AP"));
        assert!(ranges.has_authentication);
        assert_eq!(ranges.dynamic_profiles.len(), 2);

        let profile = &ranges.dynamic_profiles[0];
        assert_eq!(profile.name, "DP1");
        assert_eq!(profile.accept, ["dhcp-v4", "pppoe"]);
        assert_eq!(profile.access_profile.as_deref(), Some("DAP"));
        assert_eq!(profile.ranges.as_deref(), Some("100-200"));

        let bare = &ranges.dynamic_profiles[1];
        assert!(bare.accept.is_empty());
        assert_eq!(bare.ranges.as_deref(), Some("any-any"));
    }

    #[test]
    fn extracts_stacked_vlan_ranges() {
        let ifaces = interfaces(&juniper::parse_braces(CONFIG));
        let stacked = ifaces[0]
            .auto_configure
            .as_ref()
            .unwrap()
            .stacked_vlan_ranges
            .as_ref()
            .unwrap();
        assert_eq!(stacked.access_profile, None);
        assert!(!stacked.has_authentication);
        assert_eq!(stacked.dynamic_profiles[0].name, "SDP");
        assert_eq!(stacked.dynamic_profiles[0].accept, ["any"]);
        assert_eq!(stacked.dynamic_profiles[0].ranges.as_deref(), Some("100-200,any"));
    }

    #[test]
    fn auto_configure_is_optional() {
        let ifaces = interfaces(&juniper::parse_braces(
            "interfaces {\n    ge-0/0/0 {\n        disable;\n    }\n}\n",
        ));
        assert!(ifaces[0].auto_configure.is_none());
    }
}
