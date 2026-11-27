//! `label-switched-path`
//! `protocols { mpls { label-switched-path <name> { to <address>; ... } } }`.
//!
//! # See also
//!
//! - [label-switched-path (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/label-switched-path-edit-protocols-mpls.html)
//! - [to](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/to-edit-protocols-mpls.html)
//! - [from (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/from-edit-protocols-mpls.html)
//! - [fast-reroute (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/fast-reroute-edit-protocols-mpls.html)
//! - [link-protection (Dynamic LSPs)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/link-protection-edit-protocols-mpls.html)
//! - [adaptive (protocols mpls)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/adaptive-edit-protocols-mpls.html)
//! - [admin-down](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/admin-down-edit-protocols-mpls.html)
//! - [metric (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/metric-edit-protocols-mpls.html)
//! - [install (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/install-edit-protocols-mpls.html)
//! - [retry-timer](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/retry-timer-edit-protocols-mpls.html)
//! - [retry-limit](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/retry-limit-edit-protocols-mpls.html)
//! - [revert-timer](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/revert-timer-edit-protocols-mpls.html)
//! - [soft-preemption (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/soft-preemption-edit-protocols-mpls.html)
//! - [entropy-label (protocols mpls)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/entropy-label-edit-protocols-mpls.html)
//! - [inter-domain](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/inter-domain-edit-protocols-mpls.html)
//! - [ldp-tunneling](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/ldp-tunneling-edit-protocols-mpls.html)
//! - [description (Protocols MPLS)](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/description-edit-protocols-mpls.html)
//! - [Basic MPLS Configuration](https://networkingdocs.hpe.com/documentation/us/en/software/junos/mpls/topics/topic-map/mpls-basic-configuration.html)

use super::lsp_path::{parse_admin_group, parse_paths, MplsAdminGroupConstraints, MplsLspPath};
use crate::{active_children, arg_of, args_of, child_block, first_word, has_leaf, list_values, toggle};
use asterust::config::ConfigLine;

/// The `fast-reroute` options of an LSP. A bare `fast-reroute;` leaves every option unset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct MplsFastReroute {
    pub bandwidth: Option<String>,
    pub bandwidth_percent: Option<String>,
    pub hop_limit: Option<String>,
    pub exclude: Vec<String>,
    pub include_all: Vec<String>,
    pub include_any: Vec<String>,
}

/// A `label-switched-path <name>`. The `auto-bandwidth`, `lsp-attributes`, `policing`, `p2mp`, `traceoptions` and
/// per-class-type `bandwidth` blocks are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct MplsLabelSwitchedPath {
    pub name: String,
    /// The egress router address.
    pub to: Option<String>,
    pub from: Option<String>,
    pub description: Option<String>,
    pub disable: bool,
    pub admin_down: bool,
    pub adaptive: bool,
    pub admin_group: MplsAdminGroupConstraints,
    /// Bits per second, as written.
    pub bandwidth: Option<String>,
    pub class_of_service: Option<String>,
    pub cross_credibility_cspf: bool,
    pub entropy_label: bool,
    pub inter_domain: bool,
    pub ldp_tunneling: bool,
    pub link_protection: bool,
    pub node_link_protection: bool,
    pub metric: Option<String>,
    pub no_cspf: bool,
    pub no_decrement_ttl: bool,
    pub no_self_ping: bool,
    /// Seconds.
    pub optimize_timer: Option<String>,
    pub preference: Option<String>,
    /// `<setup> <reservation>`, as written.
    pub priority: Option<String>,
    /// `random`, `least-fill` or `most-fill`.
    pub path_selection: Option<String>,
    /// `Some(true)` for `record`, `Some(false)` for `no-record`.
    pub record: Option<bool>,
    pub retry_limit: Option<String>,
    /// Seconds.
    pub retry_timer: Option<String>,
    /// Seconds.
    pub revert_timer: Option<String>,
    pub soft_preemption: bool,
    pub standby: bool,
    /// The prefixes of `install`, which are installed in `inet.3` while the LSP is up.
    pub install: Vec<String>,
    pub fast_reroute: Option<MplsFastReroute>,
    pub primary: Vec<MplsLspPath>,
    pub secondary: Vec<MplsLspPath>,
}

pub(super) fn parse_label_switched_path(name: &str, lsp: &ConfigLine) -> MplsLabelSwitchedPath {
    let field = |keyword: &str| arg_of(lsp, keyword).map(str::to_string);
    let selection = ["random", "least-fill", "most-fill"]
        .into_iter()
        .find(|option| has_leaf(lsp, option));
    MplsLabelSwitchedPath {
        name: name.to_string(),
        to: field("to"),
        from: field("from"),
        description: field("description"),
        disable: has_leaf(lsp, "disable"),
        admin_down: has_leaf(lsp, "admin-down"),
        adaptive: has_leaf(lsp, "adaptive"),
        admin_group: parse_admin_group(lsp),
        bandwidth: field("bandwidth"),
        class_of_service: field("class-of-service"),
        cross_credibility_cspf: has_leaf(lsp, "cross-credibility-cspf"),
        entropy_label: has_leaf(lsp, "entropy-label"),
        inter_domain: has_leaf(lsp, "inter-domain"),
        ldp_tunneling: has_leaf(lsp, "ldp-tunneling"),
        link_protection: has_leaf(lsp, "link-protection"),
        node_link_protection: has_leaf(lsp, "node-link-protection"),
        metric: field("metric"),
        no_cspf: has_leaf(lsp, "no-cspf"),
        no_decrement_ttl: has_leaf(lsp, "no-decrement-ttl"),
        no_self_ping: has_leaf(lsp, "no-self-ping"),
        optimize_timer: field("optimize-timer"),
        preference: field("preference"),
        priority: field("priority"),
        path_selection: selection.map(str::to_string),
        record: toggle(lsp, "record"),
        retry_limit: field("retry-limit"),
        retry_timer: field("retry-timer"),
        revert_timer: field("revert-timer"),
        soft_preemption: has_leaf(lsp, "soft-preemption"),
        standby: has_leaf(lsp, "standby"),
        install: args_of(lsp, "install")
            .into_iter()
            .filter_map(first_word)
            .map(str::to_string)
            .collect(),
        fast_reroute: child_block(lsp, "fast-reroute").map(parse_fast_reroute),
        primary: parse_paths(lsp, "primary"),
        secondary: parse_paths(lsp, "secondary"),
    }
}

fn parse_fast_reroute(block: &ConfigLine) -> MplsFastReroute {
    let field = |keyword: &str| arg_of(block, keyword).map(str::to_string);
    let list = |keyword: &str| -> Vec<String> {
        active_children(block)
            .filter_map(|c| c.arg(keyword))
            .flat_map(list_values)
            .collect()
    };
    MplsFastReroute {
        bandwidth: field("bandwidth"),
        bandwidth_percent: field("bandwidth-percent"),
        hop_limit: field("hop-limit"),
        exclude: list("exclude"),
        include_all: list("include-all"),
        include_any: list("include-any"),
    }
}
