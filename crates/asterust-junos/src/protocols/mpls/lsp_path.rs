//! `primary` and `secondary`
//! `protocols { mpls { label-switched-path <name> { primary <path> { ... } secondary <path> { ... } } } }`.
//!
//! # See also
//!
//! - [primary (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/primary-edit-protocols-mpls.html)
//! - [secondary (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/secondary-edit-protocols-mpls.html)
//! - [standby](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/standby-edit-protocols-mpls.html)
//! - [select](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/select-edit-protocols-mpls.html)
//! - [record](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/record-edit-protocols-mpls.html)
//! - [hop-limit](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/hop-limit-edit-protocols-mpls.html)
//! - [bandwidth (Fast Reroute, Signaled, and Multiclass LSPs)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/bandwidth-edit-protocols-mpls.html)
//! - [preference (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/preference-edit-protocols-mpls.html)
//! - [priority (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/priority-edit-protocols-mpls.html)

use crate::{active_children, arg_of, args_of, has_leaf, list_values, split_name, toggle};
use asterust::config::ConfigLine;

/// The `admin-group` constraints of an LSP or one of its paths: the administrative group names it may or may not use.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct MplsAdminGroupConstraints {
    pub exclude: Vec<String>,
    pub include_all: Vec<String>,
    pub include_any: Vec<String>,
}

/// A `primary <path>` or `secondary <path>` of a label-switched path. The name refers to a `path` definition.
/// `bandwidth` is kept as written and its `ct0` to `ct3` class types are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct MplsLspPath {
    pub name: String,
    pub adaptive: bool,
    pub admin_group: MplsAdminGroupConstraints,
    pub bandwidth: Option<String>,
    pub class_of_service: Option<String>,
    pub hop_limit: Option<String>,
    pub no_cspf: bool,
    pub no_decrement_ttl: bool,
    /// Seconds.
    pub optimize_timer: Option<String>,
    pub preference: Option<String>,
    /// `<setup> <reservation>`, as written.
    pub priority: Option<String>,
    /// `Some(true)` for `record`, `Some(false)` for `no-record`.
    pub record: Option<bool>,
    /// `manual` or `unconditional`.
    pub select: Option<String>,
    pub standby: bool,
}

pub(super) fn parse_admin_group(line: &ConfigLine) -> MplsAdminGroupConstraints {
    let Some(block) = active_children(line).find(|c| c.text == "admin-group") else {
        return MplsAdminGroupConstraints::default();
    };
    let list = |keyword: &str| -> Vec<String> { args_of(block, keyword).into_iter().flat_map(list_values).collect() };
    MplsAdminGroupConstraints {
        exclude: list("exclude"),
        include_all: list("include-all"),
        include_any: list("include-any"),
    }
}

/// The `<keyword> <path>` entries of an LSP.
pub(super) fn parse_paths(lsp: &ConfigLine, keyword: &str) -> Vec<MplsLspPath> {
    crate::named_children(lsp, keyword)
        .map(|(arg, path)| parse_path(arg, path))
        .collect()
}

fn parse_path(arg: &str, path: &ConfigLine) -> MplsLspPath {
    let (name, rest) = split_name(arg);
    let field = |keyword: &str| arg_of(path, keyword).map(str::to_string);
    MplsLspPath {
        name: name.to_string(),
        adaptive: has_leaf(path, "adaptive"),
        admin_group: parse_admin_group(path),
        bandwidth: field("bandwidth"),
        class_of_service: field("class-of-service"),
        hop_limit: field("hop-limit"),
        no_cspf: has_leaf(path, "no-cspf"),
        no_decrement_ttl: has_leaf(path, "no-decrement-ttl"),
        optimize_timer: field("optimize-timer"),
        preference: field("preference"),
        priority: field("priority"),
        record: toggle(path, "record"),
        select: field("select"),
        standby: has_leaf(path, "standby") || rest.split_whitespace().any(|word| word == "standby"),
    }
}
