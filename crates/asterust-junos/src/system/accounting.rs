//! `accounting`
//! `system { accounting { ... } }`.
//!
//! # See also
//!
//! - [accounting (System)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/accounting-edit-system.html)

use super::radius_server::radius_servers;
use super::tacplus_server::tacplus_servers;
use super::{RadiusServer, TacplusServer};
use crate::{arg_of, args_of, child_block, list_values};
use asterust::config::ConfigLine;

/// The `destination` block of `system accounting`. Server secrets are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AccountingDestination {
    pub radius: Vec<RadiusServer>,
    pub tacplus: Vec<TacplusServer>,
}

/// The `system accounting` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Accounting {
    /// `change-log`, `interactive-commands` and `login`.
    pub events: Vec<String>,
    pub enhanced_avs_max: Option<String>,
    pub destination: Option<AccountingDestination>,
}

pub(super) fn parse_accounting(accounting: &ConfigLine) -> Accounting {
    Accounting {
        events: args_of(accounting, "events")
            .into_iter()
            .flat_map(list_values)
            .collect(),
        enhanced_avs_max: arg_of(accounting, "enhanced-avs-max").map(str::to_string),
        destination: child_block(accounting, "destination").map(|destination| AccountingDestination {
            radius: child_block(destination, "radius")
                .map(|radius| radius_servers(radius, "server"))
                .unwrap_or_default(),
            tacplus: child_block(destination, "tacplus")
                .map(|tacplus| tacplus_servers(tacplus, "server"))
                .unwrap_or_default(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax page. See the README "Sources".
    const ACCOUNTING: &str = r#"
system {
    accounting {
        events [ login change-log interactive-commands ];
        enhanced-avs-max 10;
        destination {
            radius {
                server 192.0.2.10 {
                    secret "$9$secret"; ## SECRET-DATA
                    accounting-port 1813;
                    timeout 5;
                    routing-instance mgmt_junos;
                }
            }
            tacplus {
                server 192.0.2.20 {
                    secret "$9$other"; ## SECRET-DATA
                    port 49;
                    single-connection;
                }
                server 192.0.2.21 {
                    secret "$9$third"; ## SECRET-DATA
                }
            }
        }
    }
}
"#;

    #[test]
    fn extracts_accounting_events() {
        let accounting = system(&juniper::parse_braces(ACCOUNTING)).unwrap().accounting.unwrap();
        assert_eq!(accounting.events, ["login", "change-log", "interactive-commands"]);
        assert_eq!(accounting.enhanced_avs_max.as_deref(), Some("10"));
    }

    #[test]
    fn extracts_accounting_destination_servers() {
        let accounting = system(&juniper::parse_braces(ACCOUNTING)).unwrap().accounting.unwrap();
        let destination = accounting.destination.unwrap();

        assert_eq!(destination.radius.len(), 1);
        assert_eq!(destination.radius[0].address, "192.0.2.10");
        assert_eq!(destination.radius[0].accounting_port.as_deref(), Some("1813"));
        assert_eq!(destination.radius[0].timeout.as_deref(), Some("5"));
        assert_eq!(destination.radius[0].routing_instance.as_deref(), Some("mgmt_junos"));

        let addresses: Vec<_> = destination.tacplus.iter().map(|s| s.address.as_str()).collect();
        assert_eq!(addresses, ["192.0.2.20", "192.0.2.21"]);
        assert!(destination.tacplus[0].single_connection);
        assert_eq!(destination.tacplus[0].port.as_deref(), Some("49"));
    }

    #[test]
    fn accounting_without_destination() {
        let accounting = system(&juniper::parse_braces(
            "system {\n    accounting {\n        events login;\n    }\n}\n",
        ))
        .unwrap()
        .accounting
        .unwrap();
        assert_eq!(accounting.events, ["login"]);
        assert!(accounting.destination.is_none());
    }

    #[test]
    fn no_accounting_is_none() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.accounting.is_none());
    }
}
