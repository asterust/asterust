//! Extracts the `autoinstallation` settings of `system`.
//!
//! Configuration path: `system { autoinstallation { ... } }`.
//!
//! # See also
//!
//! - [autoinstallation](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/autoinstallation-edit-system.html)

use super::archival::redact_password;
use crate::{active_children, child_block, has_leaf, unquote};
use asterust::config::ConfigLine;

/// An interface listed under `autoinstallation interfaces`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AutoinstallationInterface {
    pub name: String,
    pub bootp: bool,
    pub rarp: bool,
}

/// The `system autoinstallation` configuration. A bare `autoinstallation;` has no servers or
/// interfaces.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Autoinstallation {
    /// Configuration server URLs, without any password. The `password` option is not extracted.
    pub configuration_servers: Vec<String>,
    pub interfaces: Vec<AutoinstallationInterface>,
    /// `usb { disable; }`.
    pub usb_disabled: bool,
}

pub(super) fn parse_autoinstallation(autoinstallation: &ConfigLine) -> Autoinstallation {
    Autoinstallation {
        configuration_servers: child_block(autoinstallation, "configuration-servers")
            .map(|servers| {
                active_children(servers)
                    .filter_map(|server| server.text.split_whitespace().next())
                    .map(|url| redact_password(unquote(url)))
                    .collect()
            })
            .unwrap_or_default(),
        interfaces: child_block(autoinstallation, "interfaces")
            .map(|interfaces| {
                active_children(interfaces)
                    .map(|interface| AutoinstallationInterface {
                        name: interface.text.clone(),
                        bootp: has_leaf(interface, "bootp"),
                        rarp: has_leaf(interface, "rarp"),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        usb_disabled: child_block(autoinstallation, "usb").is_some_and(|usb| has_leaf(usb, "disable")),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax page. See the README "Sources".
    #[test]
    fn extracts_autoinstallation_settings() {
        let autoinstallation = system(&juniper::parse_braces(
            r#"
system {
    autoinstallation {
        configuration-servers {
            tftp://192.0.2.10/config;
            "ftp://svc:hunter2@192.0.2.11/config" {
                password "$9$secret"; ## SECRET-DATA
            }
        }
        interfaces {
            ge-0/0/0 {
                bootp;
            }
            ge-0/0/1 {
                bootp;
                rarp;
            }
        }
        usb {
            disable;
        }
    }
}
"#,
        ))
        .unwrap()
        .autoinstallation
        .unwrap();
        assert_eq!(
            autoinstallation.configuration_servers,
            ["tftp://192.0.2.10/config", "ftp://svc@192.0.2.11/config"]
        );
        assert_eq!(autoinstallation.interfaces.len(), 2);
        assert_eq!(autoinstallation.interfaces[0].name, "ge-0/0/0");
        assert!(autoinstallation.interfaces[0].bootp);
        assert!(!autoinstallation.interfaces[0].rarp);
        assert!(autoinstallation.interfaces[1].rarp);
        assert!(autoinstallation.usb_disabled);
    }

    #[test]
    fn bare_autoinstallation_is_enabled_with_no_settings() {
        let autoinstallation = system(&juniper::parse_braces("system {\n    autoinstallation;\n}\n"))
            .unwrap()
            .autoinstallation
            .unwrap();
        assert!(autoinstallation.configuration_servers.is_empty());
        assert!(autoinstallation.interfaces.is_empty());
        assert!(!autoinstallation.usb_disabled);
    }

    #[test]
    fn no_autoinstallation_is_none() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.autoinstallation.is_none());
    }
}
