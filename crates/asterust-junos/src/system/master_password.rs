//! Extracts the `master-password` settings of `system`.
//!
//! Configuration path: `system { master-password { ... } }`.
//!
//! # See also
//!
//! - [master-password](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/master-password-edit-system.html)

use crate::arg_of;
use asterust::config::ConfigLine;

/// The `system master-password` settings. The password itself is not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct MasterPassword {
    pub iteration_count: Option<String>,
    /// `hmac-sha1`, `hmac-sha2-256` or `hmac-sha2-512`.
    pub pseudorandom_function: Option<String>,
}

pub(super) fn parse_master_password(master_password: &ConfigLine) -> MasterPassword {
    MasterPassword {
        iteration_count: arg_of(master_password, "iteration-count").map(str::to_string),
        pseudorandom_function: arg_of(master_password, "pseudorandom-function").map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system;
    use asterust::config::juniper;

    #[test]
    fn extracts_master_password_settings() {
        let master = system(&juniper::parse_braces(
            "system {\n    master-password {\n        iteration-count 500;\n        pseudorandom-function hmac-sha2-256;\n    }\n}\n",
        ))
        .unwrap()
        .master_password
        .unwrap();
        assert_eq!(master.iteration_count.as_deref(), Some("500"));
        assert_eq!(master.pseudorandom_function.as_deref(), Some("hmac-sha2-256"));
    }
}
