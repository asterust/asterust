//! Typed views of common JunOS config statements, extracted from a tree parsed by
//! [`asterust::config::juniper`] (brace-delimited `show configuration` output).
//!
//! JunOS nests everything under a few top-level blocks (`interfaces`, `protocols`, `routing-options`,
//! `routing-instances`), so each module exposes a free function that reads the relevant block from a
//! [`asterust::config::Config`] rather than a per-block constructor. [`JunosDevice`] bundles a parsed
//! config with all of them as getters.
//!
//! Statements deactivated with `inactive:` are skipped. `| display set` output is not supported.

pub mod bgp;
pub mod device;
pub mod interface;
pub mod isis;
pub mod ospf;
pub mod routing_instance;
pub mod routing_options;

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

/// Active children written `<keyword> <name>` (as a block or a leaf), with the name.
pub(crate) fn named_children<'a>(
    line: &'a ConfigLine,
    keyword: &'a str,
) -> impl Iterator<Item = (&'a str, &'a ConfigLine)> {
    active_children(line).filter_map(move |c| Some((c.arg(keyword)?, c)))
}

/// The active child that is exactly `name`.
pub(crate) fn child_block<'a>(line: &'a ConfigLine, name: &str) -> Option<&'a ConfigLine> {
    active_children(line).find(|c| c.text == name)
}

/// The top-level block called `name`.
pub(crate) fn top_level<'a>(config: &'a Config, name: &str) -> Option<&'a ConfigLine> {
    config.blocks.iter().map(|b| b.get_root_line()).find(|l| l.text == name)
}

/// Values of a JunOS list: `[ a b ]` or a single bare value.
pub(crate) fn list_values(arg: &str) -> Vec<String> {
    arg.trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

/// First whitespace-separated word of `arg`.
pub(crate) fn first_word(arg: &str) -> Option<&str> {
    arg.split_whitespace().next()
}
