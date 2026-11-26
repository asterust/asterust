//! `system { login { class <name> { ... } } }`.

use super::list;
use crate::{arg_of, args_of, list_values};
use asterust::config::ConfigLine;

/// A login `class`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LoginClass {
    pub name: String,
    pub permissions: Vec<String>,
    /// Minutes.
    pub idle_timeout: Option<String>,
    /// The regular expressions as written, e.g. `"(show)|(ping)"`.
    pub allow_commands: Option<String>,
    pub deny_commands: Option<String>,
    pub allow_configuration: Option<String>,
    pub deny_configuration: Option<String>,
    /// `hh:mm`.
    pub access_start: Option<String>,
    pub access_end: Option<String>,
    pub allowed_days: Vec<String>,
    pub allow_sources: Vec<String>,
    pub deny_sources: Vec<String>,
    pub allow_times: Vec<String>,
    pub deny_times: Vec<String>,
}

pub(super) fn parse_class(name: &str, class: &ConfigLine) -> LoginClass {
    LoginClass {
        name: name.to_string(),
        permissions: args_of(class, "permissions")
            .into_iter()
            .flat_map(list_values)
            .collect(),
        idle_timeout: arg_of(class, "idle-timeout").map(str::to_string),
        allow_commands: arg_of(class, "allow-commands").map(str::to_string),
        deny_commands: arg_of(class, "deny-commands").map(str::to_string),
        allow_configuration: arg_of(class, "allow-configuration").map(str::to_string),
        deny_configuration: arg_of(class, "deny-configuration").map(str::to_string),
        access_start: arg_of(class, "access-start").map(str::to_string),
        access_end: arg_of(class, "access-end").map(str::to_string),
        allowed_days: list(class, "allowed-days"),
        allow_sources: list(class, "allow-sources"),
        deny_sources: list(class, "deny-sources"),
        allow_times: list(class, "allow-times"),
        deny_times: list(class, "deny-times"),
    }
}
