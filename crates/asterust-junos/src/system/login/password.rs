//! `password`
//! `system { login { password { ... } } }`.
//!
//! # See also
//!
//! - [password (Login)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/password-edit-system-login.html)

use crate::arg_of;
use asterust::config::ConfigLine;

/// The `system login password` policy.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LoginPassword {
    /// `character-sets` or `set-transitions`.
    pub change_type: Option<String>,
    /// `sha256` or `sha512`.
    pub format: Option<String>,
    pub maximum_length: Option<String>,
    pub minimum_length: Option<String>,
    pub maximum_lifetime: Option<String>,
    pub minimum_lifetime: Option<String>,
    pub minimum_changes: Option<String>,
    pub minimum_character_changes: Option<String>,
    pub minimum_lower_cases: Option<String>,
    pub minimum_upper_cases: Option<String>,
    pub minimum_numerics: Option<String>,
    pub minimum_punctuations: Option<String>,
    pub minimum_reuse: Option<String>,
}

pub(super) fn parse_password(password: &ConfigLine) -> LoginPassword {
    let field = |keyword: &str| arg_of(password, keyword).map(str::to_string);
    LoginPassword {
        change_type: field("change-type"),
        format: field("format"),
        maximum_length: field("maximum-length"),
        minimum_length: field("minimum-length"),
        maximum_lifetime: field("maximum-lifetime"),
        minimum_lifetime: field("minimum-lifetime"),
        minimum_changes: field("minimum-changes"),
        minimum_character_changes: field("minimum-character-changes"),
        minimum_lower_cases: field("minimum-lower-cases"),
        minimum_upper_cases: field("minimum-upper-cases"),
        minimum_numerics: field("minimum-numerics"),
        minimum_punctuations: field("minimum-punctuations"),
        minimum_reuse: field("minimum-reuse"),
    }
}
