//! `root-authentication`
//! `system { root-authentication { ... } }`.
//!
//! # See also
//!
//! - [root-authentication](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/root-authentication-edit-system.html)

use super::authentication_methods;
use crate::child_block;
use asterust::config::ConfigLine;

/// The `system root-authentication` methods. Passwords and keys are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct RootAuthentication {
    /// One entry per statement, e.g. `"encrypted-password"`, `"ssh-rsa"`, `"no-public-keys"`.
    pub methods: Vec<String>,
}

pub(super) fn root_authentication(system: &ConfigLine) -> Option<RootAuthentication> {
    child_block(system, "root-authentication").map(|auth| RootAuthentication {
        methods: authentication_methods(auth),
    })
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    #[test]
    fn extracts_root_authentication_methods_without_secrets() {
        let system = system(&juniper::parse_braces(
            r#"
system {
    root-authentication {
        encrypted-password "$6$salt$hash"; ## SECRET-DATA
        ssh-rsa "ssh-rsa AAAAB3Nza root@host"; ## SECRET-DATA
        ssh-rsa "ssh-rsa AAAAB3Nzb ops@host" {
            from 192.0.2.0/24;
        }
    }
}
"#,
        ))
        .unwrap();
        let root = system.root_authentication.unwrap();
        assert_eq!(root.methods, ["encrypted-password", "ssh-rsa", "ssh-rsa"]);
    }

    #[test]
    fn root_authentication_can_disable_public_keys() {
        let system = system(&juniper::parse_braces(
            "system {\n    root-authentication {\n        no-public-keys;\n    }\n}\n",
        ))
        .unwrap();
        assert_eq!(system.root_authentication.unwrap().methods, ["no-public-keys"]);
    }

    #[test]
    fn no_root_authentication_is_none() {
        let system = system(&juniper::parse_braces("system {\n    host-name pe1;\n}\n")).unwrap();
        assert!(system.root_authentication.is_none());
    }
}
