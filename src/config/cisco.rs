//! Cisco-style indentation-based hierarchy: a line attaches to the most
//! recent line with a strictly lower indent level, and blocks are
//! terminated by a blank line or a lone `!`.
//!
//! Covers IOS, IOS-XE, IOS-XR, NX-OS, and similar CLIs (e.g. Arista EOS,
//! FRRouting's `vtysh`) that share this convention.

use super::{add_child_at, line_at, Config, ConfigLine};

/// Parse raw `show running-config`-style text into a [`Config`].
pub fn parse(config_text: &str) -> Config {
    Config::from_root_lines(parse_lines_to_hierarchy(config_text.trim()))
}

/// Indentation level of a line: count of leading spaces, with each
/// leading tab counted as 4 spaces.
fn indent_level(line: &str) -> usize {
    let mut indent = 0;
    for ch in line.chars() {
        match ch {
            ' ' => indent += 1,
            '\t' => indent += 4,
            _ => break,
        }
    }
    indent
}

/// Build a tree of [`ConfigLine`]s from raw text, using indentation to
/// determine parent/child relationships (a line attaches to the most
/// recent line with a strictly lower indent level).
fn parse_lines_to_hierarchy(config_text: &str) -> Vec<ConfigLine> {
    let mut root_lines: Vec<ConfigLine> = Vec::new();
    // Each entry is a path from a root line down to the current insertion
    // point, as indices: stack[0] indexes into root_lines, stack[1] indexes
    // into stack[0]'s children, and so on.
    let mut stack: Vec<usize> = Vec::new();

    for (line_num, raw_line) in config_text.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed == "!" {
            continue;
        }

        let indent = indent_level(raw_line);
        let new_line = ConfigLine::new(trimmed, indent, line_num);

        if indent == 0 {
            root_lines.push(new_line);
            stack = vec![root_lines.len() - 1];
            continue;
        }

        // Pop back up the stack until we find a line with a strictly lower
        // indent level than the new line (i.e. its parent).
        while !stack.is_empty() && line_at(&root_lines, &stack).indent_level >= indent {
            stack.pop();
        }

        if stack.is_empty() {
            // No valid parent (e.g. malformed/under-indented input); treat
            // as a new root line rather than dropping it.
            root_lines.push(new_line);
            stack = vec![root_lines.len() - 1];
            continue;
        }

        let child_idx = add_child_at(&mut root_lines, &stack, new_line);
        stack.push(child_idx);
    }

    root_lines
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indent_level_counts_tabs_as_four_spaces() {
        assert_eq!(indent_level("router ospf 1"), 0);
        assert_eq!(indent_level(" router-id 10.0.0.1"), 1);
        assert_eq!(indent_level("  area 0"), 2);
        assert_eq!(indent_level("   interface Gi0/0/0/0"), 3);
        assert_eq!(indent_level("\trouter isis"), 4);
    }

    #[test]
    fn parses_simple_config() {
        let config = parse(
            "
router ospf 1
 router-id 10.0.0.1
 area 0
!
",
        );
        assert_eq!(config.get_block_count(), 1);
        assert_eq!(config.blocks[0].get_root_line().text, "router ospf 1");
    }

    #[test]
    fn parses_multiple_top_level_blocks() {
        let config = parse(
            "
router ospf 1
 router-id 10.0.0.1
!
router isis CORE
 net 49.0001.0000.0000.0001.00
!
",
        );
        assert_eq!(config.blocks.len(), 2);
        assert_eq!(config.blocks[0].get_root_line().text, "router ospf 1");
        assert_eq!(config.blocks[1].get_root_line().text, "router isis CORE");
    }

    #[test]
    fn parses_deeply_nested_hierarchy() {
        let config = parse(
            "
router ospf 1
 router-id 10.0.0.1
 area 0
  interface GigabitEthernet0/0/0/0
   cost 10
!
",
        );
        let block = &config.blocks[0];
        let iface = block.find_line_with("interface").unwrap();
        assert_eq!(iface.children.len(), 1);
        assert_eq!(iface.children[0].text, "cost 10");
    }

    #[test]
    fn find_blocks_by_pattern() {
        let config = parse(
            "
router ospf 1
 router-id 10.0.0.1
!
router isis CORE
 net 49.0001.0000.0000.0001.00
!
",
        );
        let ospf_blocks = config.find_blocks_by_pattern(r"router ospf");
        assert_eq!(ospf_blocks.len(), 1);
        assert_eq!(ospf_blocks[0].get_root_line().text, "router ospf 1");
    }
}
