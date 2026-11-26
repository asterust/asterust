//! `location`
//! `system { location { ... } }`.
//!
//! # See also
//!
//! - [location (System)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/location-edit-system.html)

use crate::arg_of;
use asterust::config::ConfigLine;

/// The `system location` block. The Bellcore coordinates (`hcoord`, `vcoord`, `lata`, `npa-nxx`)
/// are not extracted.
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
    fn no_location_is_none() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.location.is_none());
    }
}
