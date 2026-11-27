//! `static-label-switched-path`
//! `protocols { mpls { static-label-switched-path <name> { ingress { ... } transit <label> { ... } bypass <name> { ... } } } }`.
//!
//! # See also
//!
//! - [static-label-switched-path](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/topics/ref/statement/static-label-switched-path-edit-protocols-mpls.html)

use crate::{
    arg_of, args_of, child_block, first_word, has_leaf, named_children, split_name, statement_words, value_after,
};
use asterust::config::ConfigLine;

/// A `node-protection bypass-name <name> next-next-label <label>` statement.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct MplsStaticNodeProtection {
    pub bypass_name: Option<String>,
    pub next_next_label: Option<String>,
}

/// The `ingress` role of a static LSP, which pushes a label onto the traffic it receives.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct MplsStaticIngress {
    pub to: Option<String>,
    pub push: Option<String>,
    /// An address, an interface name or `<address>/<interface>`, as written.
    pub next_hop: Option<String>,
    pub bandwidth: Option<String>,
    pub class_of_service: Option<String>,
    pub description: Option<String>,
    pub metric: Option<String>,
    pub preference: Option<String>,
    pub no_install_to_address: bool,
    /// The prefixes of `install`.
    pub install: Vec<String>,
    /// The `bypass-name` of `link-protection`.
    pub link_protection: Option<String>,
    pub node_protection: Option<MplsStaticNodeProtection>,
}

/// A `transit <incoming-label>` role of a static LSP.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct MplsStaticTransit {
    pub incoming_label: String,
    pub pop: bool,
    /// The label of `swap`.
    pub swap: Option<String>,
    pub next_hop: Option<String>,
    pub bandwidth: Option<String>,
    pub description: Option<String>,
    pub link_protection: Option<String>,
    pub node_protection: Option<MplsStaticNodeProtection>,
}

/// A `bypass <name>` of a static LSP.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct MplsStaticBypass {
    pub name: String,
    pub to: Option<String>,
    pub next_hop: Option<String>,
    /// `<table> push <label>`, as written.
    pub next_table: Option<String>,
    pub bandwidth: Option<String>,
    pub description: Option<String>,
}

/// A `static-label-switched-path <name>`. The `policing` blocks are not extracted.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct MplsStaticLsp {
    pub name: String,
    pub ingress: Option<MplsStaticIngress>,
    pub transit: Vec<MplsStaticTransit>,
    pub bypasses: Vec<MplsStaticBypass>,
}

pub(super) fn parse_static_lsp(name: &str, lsp: &ConfigLine) -> MplsStaticLsp {
    MplsStaticLsp {
        name: name.to_string(),
        ingress: child_block(lsp, "ingress").map(parse_ingress),
        transit: named_children(lsp, "transit")
            .map(|(label, transit)| parse_transit(label, transit))
            .collect(),
        bypasses: named_children(lsp, "bypass")
            .map(|(name, bypass)| parse_bypass(name, bypass))
            .collect(),
    }
}

fn parse_ingress(ingress: &ConfigLine) -> MplsStaticIngress {
    let field = |keyword: &str| arg_of(ingress, keyword).map(str::to_string);
    MplsStaticIngress {
        to: field("to"),
        push: field("push"),
        next_hop: field("next-hop"),
        bandwidth: field("bandwidth"),
        class_of_service: field("class-of-service"),
        description: field("description"),
        metric: field("metric"),
        preference: field("preference"),
        no_install_to_address: has_leaf(ingress, "no-install-to-address"),
        install: args_of(ingress, "install")
            .into_iter()
            .filter_map(first_word)
            .map(str::to_string)
            .collect(),
        link_protection: parse_link_protection(ingress),
        node_protection: parse_node_protection(ingress),
    }
}

fn parse_transit(label: &str, transit: &ConfigLine) -> MplsStaticTransit {
    let field = |keyword: &str| arg_of(transit, keyword).map(str::to_string);
    MplsStaticTransit {
        incoming_label: split_name(label).0.to_string(),
        pop: has_leaf(transit, "pop"),
        swap: field("swap"),
        next_hop: field("next-hop"),
        bandwidth: field("bandwidth"),
        description: field("description"),
        link_protection: parse_link_protection(transit),
        node_protection: parse_node_protection(transit),
    }
}

fn parse_bypass(name: &str, bypass: &ConfigLine) -> MplsStaticBypass {
    let field = |keyword: &str| arg_of(bypass, keyword).map(str::to_string);
    MplsStaticBypass {
        name: name.to_string(),
        to: field("to"),
        next_hop: field("next-hop"),
        next_table: field("next-table"),
        bandwidth: field("bandwidth"),
        description: field("description"),
    }
}

/// `link-protection bypass-name <name>`, on one line or with the option as a child.
fn parse_link_protection(role: &ConfigLine) -> Option<String> {
    statement_words(role, "link-protection").and_then(|words| value_after(&words, "bypass-name"))
}

fn parse_node_protection(role: &ConfigLine) -> Option<MplsStaticNodeProtection> {
    let words = statement_words(role, "node-protection")?;
    Some(MplsStaticNodeProtection {
        bypass_name: value_after(&words, "bypass-name"),
        next_next_label: value_after(&words, "next-next-label"),
    })
}
