//! Extracts the `local-policy-decision-function` settings of `system services`.
//!
//! Configuration path: `system { services { local-policy-decision-function { ... } } }`.
//!
//! # See also
//!
//! - [`[edit system]` hierarchy](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/topic-map/hierarchy-edit-system.html)

use crate::{active_children, arg_of, child_block, named_children};
use asterust::config::ConfigLine;

/// An `aacl-statistics-profile <name>`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AaclStatisticsProfile {
    pub name: String,
    pub aacl_fields: Vec<String>,
    pub file: Option<String>,
    /// `interim-active-only` or `interim-full`.
    pub record_mode: Option<String>,
    /// Minutes.
    pub report_interval: Option<String>,
}

/// The `system services local-policy-decision-function` settings.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct LocalPolicyDecisionFunction {
    /// Profiles from `statistics`.
    pub statistics: Vec<AaclStatisticsProfile>,
}

pub(super) fn parse_local_policy_decision_function(function: &ConfigLine) -> LocalPolicyDecisionFunction {
    LocalPolicyDecisionFunction {
        statistics: child_block(function, "statistics")
            .map(|statistics| {
                named_children(statistics, "aacl-statistics-profile")
                    .map(|(name, profile)| AaclStatisticsProfile {
                        name: name.to_string(),
                        aacl_fields: child_block(profile, "aacl-fields")
                            .map(|fields| active_children(fields).map(|f| f.text.clone()).collect())
                            .unwrap_or_default(),
                        file: arg_of(profile, "file").map(str::to_string),
                        record_mode: arg_of(profile, "record-mode").map(str::to_string),
                        report_interval: arg_of(profile, "report-interval").map(str::to_string),
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }
}
