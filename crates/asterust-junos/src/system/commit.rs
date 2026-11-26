//! `system { commit { ... } }`.

use crate::has_leaf;
use asterust::config::ConfigLine;

/// The `system commit` options. `peers`, which carries user credentials, is not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Commit {
    pub synchronize: bool,
    pub fast_synchronize: bool,
    pub peers_synchronize: bool,
    pub commit_synchronize_server: bool,
    pub config_sync_with_scp: bool,
    pub delta_export: bool,
    pub force_commit_log: bool,
    pub notification: bool,
    pub server: bool,
    /// `Some(true)` for `persist-groups-inheritance`, `Some(false)` for `no-persist-groups-inheritance`.
    pub persist_groups_inheritance: Option<bool>,
}

pub(super) fn parse_commit(commit: &ConfigLine) -> Commit {
    let persist_groups_inheritance = if has_leaf(commit, "persist-groups-inheritance") {
        Some(true)
    } else if has_leaf(commit, "no-persist-groups-inheritance") {
        Some(false)
    } else {
        None
    };
    Commit {
        synchronize: has_leaf(commit, "synchronize"),
        fast_synchronize: has_leaf(commit, "fast-synchronize"),
        peers_synchronize: has_leaf(commit, "peers-synchronize"),
        commit_synchronize_server: has_leaf(commit, "commit-synchronize-server"),
        config_sync_with_scp: has_leaf(commit, "config-sync-with-scp"),
        delta_export: has_leaf(commit, "delta-export"),
        force_commit_log: has_leaf(commit, "force-commit-log"),
        notification: has_leaf(commit, "notification"),
        server: has_leaf(commit, "server"),
        persist_groups_inheritance,
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    // Not from documentation: written from the CLI Reference syntax page. See the README "Sources".
    #[test]
    fn extracts_commit_options() {
        let commit = system(&juniper::parse_braces(
            r#"
system {
    commit {
        synchronize;
        fast-synchronize;
        peers-synchronize;
        commit-synchronize-server;
        config-sync-with-scp;
        delta-export;
        force-commit-log;
        notification;
        server;
        persist-groups-inheritance;
    }
}
"#,
        ))
        .unwrap()
        .commit
        .unwrap();
        assert!(commit.synchronize);
        assert!(commit.fast_synchronize);
        assert!(commit.peers_synchronize);
        assert!(commit.commit_synchronize_server);
        assert!(commit.config_sync_with_scp);
        assert!(commit.delta_export);
        assert!(commit.force_commit_log);
        assert!(commit.notification);
        assert!(commit.server);
        assert_eq!(commit.persist_groups_inheritance, Some(true));
    }

    #[test]
    fn commit_options_default_to_off() {
        let commit = system(&juniper::parse_braces(
            "system {\n    commit {\n        no-persist-groups-inheritance;\n    }\n}\n",
        ))
        .unwrap()
        .commit
        .unwrap();
        assert!(!commit.synchronize);
        assert!(!commit.delta_export);
        assert_eq!(commit.persist_groups_inheritance, Some(false));
    }

    #[test]
    fn unset_persist_groups_inheritance_is_none() {
        let commit = system(&juniper::parse_braces(
            "system {\n    commit {\n        synchronize;\n    }\n}\n",
        ))
        .unwrap()
        .commit
        .unwrap();
        assert_eq!(commit.persist_groups_inheritance, None);
    }

    #[test]
    fn no_commit_is_none() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.commit.is_none());
    }
}
