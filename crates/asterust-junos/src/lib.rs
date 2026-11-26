//! Typed views of common JunOS config statements, extracted from a tree parsed by
//! [`asterust::config::juniper`] (brace-delimited `show configuration` output).
//!
//! JunOS nests everything under a few top-level blocks (`interfaces`, `protocols`,
//! `routing-options`, `routing-instances`, `policy-options`, `firewall`, `system`), so each module
//! exposes a free function that reads the relevant block from a [`asterust::config::Config`] rather
//! than a per-block constructor. [`JunosDevice`] bundles a parsed config with all of them as
//! getters.
//!
//! The module tree mirrors the configuration hierarchy: `system { radius-server ... }` is
//! `system::radius_server` and `protocols { bgp { ... } }` is `protocols::bgp`. A statement with
//! many children of its own gets a folder, so `system::services::ssh` models the `ssh` statement
//! of `system services`.
//!
//! Statements deactivated with `inactive:` are skipped. `| display set` output is not supported.
//!
//! Each module documents its configuration path and links to the matching pages of the Junos CLI
//! Reference.
//!
//! # Examples
//!
//! ```
//! use asterust_junos::JunosDevice;
//!
//! let device = JunosDevice::parse("interfaces {\n    lo0 {\n        unit 0;\n    }\n}\n");
//! assert_eq!(device.interfaces()[0].name, "lo0");
//! ```
//!
//! # See also
//!
//! - [Junos CLI Reference](https://networkingdocs.hpe.com/documentation/us/en/software/junos/cli-reference/index.html)

pub mod device;
pub mod firewall;
pub mod interfaces;
pub mod policy_options;
pub mod protocols;
pub mod routing_instances;
pub mod routing_options;
pub mod system;

pub use device::JunosDevice;

use asterust::config::{Config, ConfigLine};

/// Drop the quotes JunOS puts around values containing spaces.
pub(crate) fn unquote(value: &str) -> &str {
    value.trim().trim_matches('"')
}

/// Direct children of `line`, minus statements deactivated with `inactive:`.
pub(crate) fn active_children(line: &ConfigLine) -> impl Iterator<Item = &ConfigLine> {
    line.children.iter().filter(|c| !c.text.starts_with("inactive:"))
}

/// Argument of the first active child that is `<keyword> <argument>`.
pub(crate) fn arg_of<'a>(line: &'a ConfigLine, keyword: &str) -> Option<&'a str> {
    active_children(line).find_map(|c| c.arg(keyword)).map(unquote)
}

/// Arguments of every active child that is `<keyword> <argument>`.
pub(crate) fn args_of<'a>(line: &'a ConfigLine, keyword: &str) -> Vec<&'a str> {
    active_children(line)
        .filter_map(|c| c.arg(keyword))
        .map(unquote)
        .collect()
}

/// Whether `line` has an active child that is exactly `name` (a flag such as `passive`).
pub(crate) fn has_leaf(line: &ConfigLine, name: &str) -> bool {
    active_children(line).any(|c| c.text == name)
}

/// `Some(true)` for `<name>`, `Some(false)` for `no-<name>`, `None` if neither is present. Either
/// may carry options.
pub(crate) fn toggle(line: &ConfigLine, name: &str) -> Option<bool> {
    let written = |statement: &str| active_children(line).any(|c| c.text == statement || c.arg(statement).is_some());
    if written(name) {
        Some(true)
    } else if written(&format!("no-{name}")) {
        Some(false)
    } else {
        None
    }
}

/// Active children written `<keyword> <name>` (as a block or a leaf), with the name.
pub(crate) fn named_children<'a>(
    line: &'a ConfigLine,
    keyword: &'a str,
) -> impl Iterator<Item = (&'a str, &'a ConfigLine)> {
    // Names can be quoted, e.g. `neighbor "fe80::20%ge-0/0/1.1";`.
    active_children(line).filter_map(move |c| Some((unquote(c.arg(keyword)?), c)))
}

/// The active child that is exactly `name`.
pub(crate) fn child_block<'a>(line: &'a ConfigLine, name: &str) -> Option<&'a ConfigLine> {
    active_children(line).find(|c| c.text == name)
}

/// The top-level block called `name`.
pub(crate) fn top_level<'a>(config: &'a Config, name: &str) -> Option<&'a ConfigLine> {
    config.blocks.iter().map(|b| b.get_root_line()).find(|l| l.text == name)
}

/// Values of a JunOS list: `[ a b ]` or a single bare value. A quoted value may contain spaces.
pub(crate) fn list_values(arg: &str) -> Vec<String> {
    let inner = arg.trim().trim_start_matches('[').trim_end_matches(']');
    let mut values = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for ch in inner.chars() {
        match ch {
            '"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => {
                if !current.is_empty() {
                    values.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        values.push(current);
    }
    values
}

/// Split `<name> <rest>`, where the name may be quoted.
pub(crate) fn split_name(arg: &str) -> (&str, &str) {
    if let Some((name, rest)) = arg.strip_prefix('"').and_then(|quoted| quoted.split_once('"')) {
        return (name, rest.trim());
    }
    arg.split_once(char::is_whitespace)
        .map_or((arg, ""), |(name, rest)| (name, rest.trim()))
}

/// Split `<keyword> <argument>` into its two parts; the argument is empty for a bare keyword.
pub(crate) fn split_keyword(statement: &str) -> (&str, &str) {
    statement
        .split_once(char::is_whitespace)
        .map_or((statement, ""), |(keyword, arg)| (keyword, arg.trim()))
}

/// Statements under `line`'s `keyword` entries, whether written `then x;` or `then { x; y; }`.
pub(crate) fn statements<'a>(line: &'a ConfigLine, keyword: &str) -> Vec<&'a str> {
    let mut found = Vec::new();
    for child in active_children(line) {
        if child.text == keyword {
            found.extend(active_children(child).map(|s| s.text.as_str()));
        } else if let Some(arg) = child.arg(keyword) {
            found.push(arg);
        }
    }
    found
}

/// Words of the first active `<keyword> ...` statement: those on its own line, then those of each
/// child statement.
pub(crate) fn statement_words<'a>(parent: &'a ConfigLine, keyword: &str) -> Option<Vec<&'a str>> {
    let statement = active_children(parent).find(|c| c.text == keyword || c.arg(keyword).is_some())?;
    Some(
        statement
            .text
            .split_whitespace()
            .skip(1)
            .chain(active_children(statement).flat_map(|c| c.text.split_whitespace()))
            .collect(),
    )
}

/// The word after the first occurrence of `name` in `words`.
pub(crate) fn value_after(words: &[&str], name: &str) -> Option<String> {
    let position = words.iter().position(|w| *w == name)?;
    words.get(position + 1).map(|w| w.to_string())
}

/// First whitespace-separated word of `arg`.
pub(crate) fn first_word(arg: &str) -> Option<&str> {
    arg.split_whitespace().next()
}
