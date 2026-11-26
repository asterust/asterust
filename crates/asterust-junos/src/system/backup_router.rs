//! `system { backup-router ... }` and `system { inet6-backup-router ... }`.

use crate::{list_values, split_name};

/// A `backup-router` or `inet6-backup-router`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BackupRouter {
    pub address: String,
    /// Empty when the router is reachable for all destinations.
    pub destinations: Vec<String>,
}

/// `<address> [destination <prefix | [ prefixes ]>]`.
pub(super) fn parse_backup_router(arg: &str) -> BackupRouter {
    let (address, rest) = split_name(arg);
    BackupRouter {
        address: address.to_string(),
        destinations: rest.strip_prefix("destination").map(list_values).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    #[test]
    fn extracts_backup_routers() {
        let system = system(&juniper::parse_braces(
            "system {\n    backup-router 192.0.2.1 destination [ 10.0.0.0/8 172.16.0.0/12 ];\n    inet6-backup-router 2001:db8::1;\n}\n",
        ))
        .unwrap();
        let backup = system.backup_router.unwrap();
        assert_eq!(backup.address, "192.0.2.1");
        assert_eq!(backup.destinations, ["10.0.0.0/8", "172.16.0.0/12"]);

        let backup6 = system.inet6_backup_router.unwrap();
        assert_eq!(backup6.address, "2001:db8::1");
        assert!(backup6.destinations.is_empty());
    }

    #[test]
    fn single_destination_backup_router() {
        let system = system(&juniper::parse_braces(
            "system {\n    backup-router 192.0.2.1 destination 10.0.0.0/8;\n}\n",
        ))
        .unwrap();
        assert_eq!(system.backup_router.unwrap().destinations, ["10.0.0.0/8"]);
    }
}
