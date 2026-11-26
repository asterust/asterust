//! Extracts the `ports` settings of `system`.
//!
//! Configuration path: `system { ports { console { ... } auxiliary { ... } } }`.
//!
//! # See also
//!
//! - [ports (System)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/ports-edit-system.html)
//! - [console (System Ports)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/console-edit-system-ports.html)
//! - [auxiliary](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/auxiliary-edit-system.html)

use crate::{arg_of, child_block, has_leaf};
use asterust::config::ConfigLine;

/// The `system ports console` block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Console {
    pub disable: bool,
    pub insecure: bool,
    pub log_out_on_disconnect: bool,
    /// From `type`, e.g. `vt100`.
    pub terminal_type: Option<String>,
}

/// The `system ports auxiliary` block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Auxiliary {
    pub disable: bool,
    pub insecure: bool,
    /// From `type`, e.g. `vt100`.
    pub terminal_type: Option<String>,
    /// `mini-usb` or `rj45`.
    pub port_type: Option<String>,
}

/// The `system ports` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Ports {
    pub console: Option<Console>,
    pub auxiliary: Option<Auxiliary>,
}

pub(super) fn parse_ports(ports: &ConfigLine) -> Ports {
    Ports {
        console: child_block(ports, "console").map(|console| Console {
            disable: has_leaf(console, "disable"),
            insecure: has_leaf(console, "insecure"),
            log_out_on_disconnect: has_leaf(console, "log-out-on-disconnect"),
            terminal_type: arg_of(console, "type").map(str::to_string),
        }),
        auxiliary: child_block(ports, "auxiliary").map(|auxiliary| Auxiliary {
            disable: has_leaf(auxiliary, "disable"),
            insecure: has_leaf(auxiliary, "insecure"),
            terminal_type: arg_of(auxiliary, "type").map(str::to_string),
            port_type: arg_of(auxiliary, "port-type").map(str::to_string),
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    #[test]
    fn extracts_console_and_auxiliary_ports() {
        let system = system(&juniper::parse_braces(
            r#"
system {
    ports {
        console {
            log-out-on-disconnect;
            type vt100;
            insecure;
        }
        auxiliary {
            disable;
            port-type rj45;
        }
    }
}
"#,
        ))
        .unwrap();
        let ports = system.ports.unwrap();

        let console = ports.console.unwrap();
        assert!(console.log_out_on_disconnect);
        assert!(console.insecure);
        assert!(!console.disable);
        assert_eq!(console.terminal_type.as_deref(), Some("vt100"));

        let auxiliary = ports.auxiliary.unwrap();
        assert!(auxiliary.disable);
        assert!(!auxiliary.insecure);
        assert_eq!(auxiliary.terminal_type, None);
        assert_eq!(auxiliary.port_type.as_deref(), Some("rj45"));
    }

    #[test]
    fn ports_may_configure_only_one_port() {
        let system = system(&juniper::parse_braces(
            "system {\n    ports {\n        console {\n            disable;\n        }\n    }\n}\n",
        ))
        .unwrap();
        let ports = system.ports.unwrap();
        assert!(ports.console.unwrap().disable);
        assert!(ports.auxiliary.is_none());
    }

    #[test]
    fn no_ports_is_none() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.ports.is_none());
    }
}
