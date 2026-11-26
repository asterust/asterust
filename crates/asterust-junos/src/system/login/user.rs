//! Extracts the `user` entries of `system login`.
//!
//! Configuration path: `system { login { user <name> { ... } } }`.
//!
//! # See also
//!
//! - [user (Access)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/user-edit-system-login.html)

use crate::system::authentication_methods;
use crate::{arg_of, child_block};
use asterust::config::ConfigLine;

/// A login `user`. Passwords and keys are not extracted, only which authentication methods are
/// present.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LoginUser {
    pub name: String,
    pub class: Option<String>,
    pub uid: Option<String>,
    pub full_name: Option<String>,
    /// One entry per authentication statement, e.g. `"encrypted-password"`, `"ssh-rsa"`.
    pub authentication_methods: Vec<String>,
}

pub(super) fn parse_user(name: &str, user: &ConfigLine) -> LoginUser {
    LoginUser {
        name: name.to_string(),
        class: arg_of(user, "class").map(str::to_string),
        uid: arg_of(user, "uid").map(str::to_string),
        full_name: arg_of(user, "full-name").map(str::to_string),
        authentication_methods: child_block(user, "authentication")
            .map(authentication_methods)
            .unwrap_or_default(),
    }
}
