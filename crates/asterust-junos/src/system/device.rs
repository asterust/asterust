//! Physical device settings under `system`: location and serial ports.

use crate::{arg_of, child_block, has_leaf};
use asterust::config::ConfigLine;

/// A `system location` block. The Bellcore coordinates (`hcoord`, `vcoord`, `lata`, `npa-nxx`) are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Location {
    pub building: Option<String>,
    pub floor: Option<String>,
    pub rack: Option<String>,
    pub country_code: Option<String>,
    pub postal_code: Option<String>,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    /// Feet above sea level.
    pub altitude: Option<String>,
}

/// A `system ports console` or `auxiliary` block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SerialPort {
    pub disable: bool,
    pub insecure: bool,
    pub log_out_on_disconnect: bool,
    /// From `type`, e.g. `vt100`.
    pub terminal_type: Option<String>,
    /// `mini-usb` or `rj45`; only on the auxiliary port.
    pub port_type: Option<String>,
}

/// The `system ports` configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Ports {
    pub console: Option<SerialPort>,
    pub auxiliary: Option<SerialPort>,
}

pub(super) fn parse_location(location: &ConfigLine) -> Location {
    let field = |keyword: &str| arg_of(location, keyword).map(str::to_string);
    Location {
        building: field("building"),
        floor: field("floor"),
        rack: field("rack"),
        country_code: field("country-code"),
        postal_code: field("postal-code"),
        latitude: field("latitude"),
        longitude: field("longitude"),
        altitude: field("altitude"),
    }
}

pub(super) fn ports(system: &ConfigLine) -> Option<Ports> {
    child_block(system, "ports").map(|ports| Ports {
        console: child_block(ports, "console").map(parse_serial_port),
        auxiliary: child_block(ports, "auxiliary").map(parse_serial_port),
    })
}

fn parse_serial_port(port: &ConfigLine) -> SerialPort {
    SerialPort {
        disable: has_leaf(port, "disable"),
        insecure: has_leaf(port, "insecure"),
        log_out_on_disconnect: has_leaf(port, "log-out-on-disconnect"),
        terminal_type: arg_of(port, "type").map(str::to_string),
        port_type: arg_of(port, "port-type").map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    #[test]
    fn extracts_location() {
        let system = system(&juniper::parse_braces(
            r#"
system {
    location {
        building "DC East";
        floor 2;
        rack 14;
        country-code US;
        postal-code 10001;
        latitude 40.7128;
        longitude -74.006;
        altitude 33;
    }
}
"#,
        ))
        .unwrap();
        let location = system.location.unwrap();
        assert_eq!(location.building.as_deref(), Some("DC East"));
        assert_eq!(location.floor.as_deref(), Some("2"));
        assert_eq!(location.rack.as_deref(), Some("14"));
        assert_eq!(location.country_code.as_deref(), Some("US"));
        assert_eq!(location.postal_code.as_deref(), Some("10001"));
        assert_eq!(location.latitude.as_deref(), Some("40.7128"));
        assert_eq!(location.longitude.as_deref(), Some("-74.006"));
        assert_eq!(location.altitude.as_deref(), Some("33"));
    }

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
        assert_eq!(console.port_type, None);

        let auxiliary = ports.auxiliary.unwrap();
        assert!(auxiliary.disable);
        assert_eq!(auxiliary.port_type.as_deref(), Some("rj45"));
    }

    #[test]
    fn extracts_configuration_limits_and_no_redirects() {
        let system = system(&juniper::parse_braces(
            "system {\n    max-configurations-on-flash 5;\n    max-configuration-rollbacks 20;\n    no-redirects;\n}\n",
        ))
        .unwrap();
        assert_eq!(system.max_configurations_on_flash.as_deref(), Some("5"));
        assert_eq!(system.max_configuration_rollbacks.as_deref(), Some("20"));
        assert!(system.no_redirects);
    }

    #[test]
    fn device_settings_are_optional() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.location.is_none());
        assert!(system.ports.is_none());
        assert_eq!(system.max_configurations_on_flash, None);
        assert_eq!(system.max_configuration_rollbacks, None);
        assert!(!system.no_redirects);
    }
}
