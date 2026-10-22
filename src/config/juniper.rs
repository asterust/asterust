//! Juniper-style brace-delimited hierarchy: nesting is defined by `{` `}`
//! pairs rather than indentation, and leaf statements end in `;`.
//!
//! Covers JunOS `show configuration` output, and other brace-based CLIs
//! that follow the same convention (e.g. VyOS/Vyatta's brace mode).
//!
//! Does not (yet) handle JunOS's flat `| display set` output, which has no
//! real nesting and doesn't fit this tree model well — see the module docs
//! for why that would need its own representation.

use super::{add_child_at, Config, ConfigLine};

/// Parse raw `show configuration`-style brace text into a [`Config`].
///
/// Comment lines (starting with `#`) are skipped. Malformed input (e.g. an
/// unmatched closing brace, or a statement with neither `{` nor `;`) is
/// handled permissively rather than rejected: unmatched `}` is ignored,
/// and a bare statement is treated as a leaf at the current depth.
pub fn parse_braces(config_text: &str) -> Config {
    Config::from_root_lines(parse_lines_to_hierarchy(config_text.trim()))
}

fn parse_lines_to_hierarchy(config_text: &str) -> Vec<ConfigLine> {
    let mut root_lines: Vec<ConfigLine> = Vec::new();
    // Path to the block currently being filled in; empty means top level.
    let mut stack: Vec<usize> = Vec::new();

    for (line_num, raw_line) in config_text.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed == "}" {
            stack.pop();
            continue;
        }

        if let Some(header) = trimmed.strip_suffix('{') {
            let new_line = ConfigLine::new(header.trim(), stack.len(), line_num);
            let idx = push_line(&mut root_lines, &stack, new_line);
            stack.push(idx);
            continue;
        }

        // Leaf statement, normally `;`-terminated; fall back to the whole
        // trimmed line if the terminator is missing rather than dropping it.
        let text = trimmed.strip_suffix(';').unwrap_or(trimmed).trim();
        let leaf = ConfigLine::new(text, stack.len(), line_num);
        push_line(&mut root_lines, &stack, leaf);
    }

    root_lines
}

/// Append `line` as a child of the block addressed by `path`, or as a new
/// root line if `path` is empty (top level). Returns the index it was
/// inserted at, relative to whichever list it landed in.
fn push_line(root_lines: &mut Vec<ConfigLine>, path: &[usize], line: ConfigLine) -> usize {
    if path.is_empty() {
        root_lines.push(line);
        root_lines.len() - 1
    } else {
        add_child_at(root_lines, path, line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flat_leaf_statements() {
        let config = parse_braces("host-name router1;\ndomain-name example.com;");
        assert_eq!(config.blocks.len(), 2);
        assert_eq!(config.blocks[0].get_root_line().text, "host-name router1");
        assert_eq!(config.blocks[1].get_root_line().text, "domain-name example.com");
    }

    #[test]
    fn parses_nested_braces() {
        let config = parse_braces(
            "
interfaces {
    ge-0/0/0 {
        unit 0 {
            family inet {
                address 10.0.0.1/24;
            }
        }
    }
}
",
        );
        assert_eq!(config.blocks.len(), 1);
        let block = &config.blocks[0];
        assert_eq!(block.get_root_line().text, "interfaces");
        let addr = block.find_line_with("address").unwrap();
        assert_eq!(addr.text, "address 10.0.0.1/24");
        assert_eq!(addr.indent_level, 4);
    }

    #[test]
    fn parses_multiple_statements_in_one_block() {
        let config = parse_braces(
            "
system {
    host-name router1;
    domain-name example.com;
}
",
        );
        let block = &config.blocks[0];
        assert_eq!(block.get_direct_children().len(), 2);
        assert_eq!(block.get_direct_children()[0].text, "host-name router1");
        assert_eq!(block.get_direct_children()[1].text, "domain-name example.com");
    }

    #[test]
    fn parses_sibling_blocks_at_same_depth() {
        let config = parse_braces(
            "
interfaces {
    ge-0/0/0 {
        unit 0;
    }
    ge-0/0/1 {
        unit 0;
    }
}
",
        );
        let block = &config.blocks[0];
        assert_eq!(block.get_direct_children().len(), 2);
        assert_eq!(block.get_direct_children()[0].text, "ge-0/0/0");
        assert_eq!(block.get_direct_children()[1].text, "ge-0/0/1");
    }

    #[test]
    fn skips_comment_lines() {
        let config = parse_braces("# top-level comment\nhost-name router1;");
        assert_eq!(config.blocks.len(), 1);
        assert_eq!(config.blocks[0].get_root_line().text, "host-name router1");
    }

    #[test]
    fn find_blocks_by_pattern() {
        let config = parse_braces(
            "
interfaces {
    ge-0/0/0;
}
protocols {
    ospf;
}
",
        );
        let matches = config.find_blocks_by_pattern("interfaces");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].get_root_line().text, "interfaces");
    }
}
