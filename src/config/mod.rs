//! Generic hierarchical configuration tree: a vendor-agnostic data model
//! ([`ConfigLine`], [`ConfigBlock`], [`Config`]) shared by multiple parsing
//! strategies, one per vendor/syntax family.
//!
//! - [`cisco`]: indentation-based hierarchy, `!`-terminated blocks
//!   (IOS, IOS-XE, IOS-XR, NX-OS, and similar "Cisco-style" CLIs).
//! - [`juniper`]: brace-delimited hierarchy (JunOS `show configuration`
//!   output).
//!
//! The types below don't know or care which parser built the tree; each
//! vendor module is responsible only for turning raw text into a
//! `Vec<ConfigLine>` of root lines.

pub mod cisco;
pub mod juniper;

use regex::Regex;

/// A single configuration line, with any nested lines as children.
/// "Nested" means different things per vendor (indentation depth for
/// Cisco, brace depth for Juniper); `indent_level` holds whatever depth
/// measure the parser that built this tree uses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigLine {
    pub text: String,
    pub indent_level: usize,
    pub line_number: usize,
    pub children: Vec<ConfigLine>,
}

impl ConfigLine {
    pub fn new(text: impl Into<String>, indent_level: usize, line_number: usize) -> Self {
        Self { text: text.into(), indent_level, line_number, children: Vec::new() }
    }

    pub fn add_child(&mut self, child: ConfigLine) {
        self.children.push(child);
    }

    /// First direct child whose text matches `pattern`.
    pub fn find_child_with(&self, pattern: &str) -> Option<&ConfigLine> {
        let re = Regex::new(pattern).ok()?;
        self.children.iter().find(|c| re.is_match(&c.text))
    }

    /// All direct children whose text matches `pattern`.
    pub fn find_all_children_with(&self, pattern: &str) -> Vec<&ConfigLine> {
        let Ok(re) = Regex::new(pattern) else { return Vec::new() };
        self.children.iter().filter(|c| re.is_match(&c.text)).collect()
    }

    /// Whether any direct child's text matches `pattern`.
    pub fn has_child_with(&self, pattern: &str) -> bool {
        self.find_child_with(pattern).is_some()
    }

    /// Match `pattern` against this line's own text and return capture
    /// group 1 as a string, or `None` if the pattern doesn't match or has
    /// no group 1.
    pub fn match_str_group(&self, pattern: &str) -> Option<String> {
        let re = Regex::new(pattern).ok()?;
        re.captures(&self.text)?.get(1).map(|m| m.as_str().to_string())
    }

    /// Match `pattern` against this line's own text and parse capture
    /// group 1 as an integer, or `None` if the pattern doesn't match, has
    /// no group 1, or the captured text isn't a valid integer.
    pub fn match_int_group(&self, pattern: &str) -> Option<i64> {
        self.match_str_group(pattern)?.parse().ok()
    }
}

/// A top-level configuration block (e.g. "router ospf 1", "interfaces")
/// and the hierarchy of lines beneath it.
#[derive(Clone, Debug)]
pub struct ConfigBlock {
    pub root_line: ConfigLine,
}

impl ConfigBlock {
    pub fn new(root_line: ConfigLine) -> Self {
        Self { root_line }
    }

    pub fn get_root_line(&self) -> &ConfigLine {
        &self.root_line
    }

    pub fn get_direct_children(&self) -> &[ConfigLine] {
        &self.root_line.children
    }

    /// Every line in the block (root plus all descendants, depth-first),
    /// recomputed on each call.
    pub fn get_all_lines(&self) -> Vec<&ConfigLine> {
        let mut out = Vec::new();
        collect_all_lines(&self.root_line, &mut out);
        out
    }

    pub fn find_child_with(&self, pattern: &str) -> Option<&ConfigLine> {
        self.root_line.find_child_with(pattern)
    }

    pub fn find_all_children_with(&self, pattern: &str) -> Vec<&ConfigLine> {
        self.root_line.find_all_children_with(pattern)
    }

    /// First line (at any depth) matching `pattern`.
    pub fn find_line_with(&self, pattern: &str) -> Option<&ConfigLine> {
        let re = Regex::new(pattern).ok()?;
        self.get_all_lines().into_iter().find(|l| re.is_match(&l.text))
    }

    /// Every line (at any depth) matching `pattern`.
    pub fn re_search_children(&self, pattern: &str) -> Vec<&ConfigLine> {
        let Ok(re) = Regex::new(pattern) else { return Vec::new() };
        self.get_all_lines().into_iter().filter(|l| re.is_match(&l.text)).collect()
    }
}

fn collect_all_lines<'a>(line: &'a ConfigLine, out: &mut Vec<&'a ConfigLine>) {
    out.push(line);
    for child in &line.children {
        collect_all_lines(child, out);
    }
}

/// Vendor parsers represent "where to insert next" as a path of child
/// indices from a root line (`path[0]` indexes `root_lines`, `path[1]`
/// indexes that line's children, and so on). These helpers do the
/// index-chasing so each parser only has to decide *when* to push/pop.
pub(crate) fn line_at<'a>(root_lines: &'a [ConfigLine], path: &[usize]) -> &'a ConfigLine {
    let mut line = &root_lines[path[0]];
    for &idx in &path[1..] {
        line = &line.children[idx];
    }
    line
}

pub(crate) fn add_child_at(root_lines: &mut [ConfigLine], path: &[usize], child: ConfigLine) -> usize {
    let mut line = &mut root_lines[path[0]];
    for &idx in &path[1..] {
        line = &mut line.children[idx];
    }
    line.add_child(child);
    line.children.len() - 1
}

/// A parsed configuration: one [`ConfigBlock`] per top-level line, however
/// "top-level" was determined by the vendor-specific parser that built it.
#[derive(Clone, Debug)]
pub struct Config {
    pub blocks: Vec<ConfigBlock>,
}

impl Config {
    /// Wrap already-built root lines (e.g. from [`cisco::parse`] or
    /// [`juniper::parse_braces`]) into a `Config`.
    pub fn from_root_lines(root_lines: Vec<ConfigLine>) -> Self {
        Self { blocks: root_lines.into_iter().map(ConfigBlock::new).collect() }
    }

    pub fn get_block_count(&self) -> usize {
        self.blocks.len()
    }

    /// All blocks whose root line matches `pattern`.
    pub fn find_blocks_by_pattern(&self, pattern: &str) -> Vec<&ConfigBlock> {
        let Ok(re) = Regex::new(pattern) else { return Vec::new() };
        self.blocks.iter().filter(|b| re.is_match(&b.root_line.text)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_config_line() {
        let line = ConfigLine::new("router ospf 1", 0, 1);
        assert_eq!(line.text, "router ospf 1");
        assert_eq!(line.indent_level, 0);
        assert_eq!(line.line_number, 1);
        assert!(line.children.is_empty());
    }

    #[test]
    fn add_child() {
        let mut parent = ConfigLine::new("router ospf 1", 0, 0);
        parent.add_child(ConfigLine::new("router-id 10.0.0.1", 1, 1));
        parent.add_child(ConfigLine::new("area 0", 1, 2));
        assert_eq!(parent.children.len(), 2);
        assert_eq!(parent.children[0].text, "router-id 10.0.0.1");
        assert_eq!(parent.children[1].text, "area 0");
    }

    #[test]
    fn find_child_with() {
        let mut parent = ConfigLine::new("router ospf 1", 0, 0);
        parent.add_child(ConfigLine::new("router-id 10.0.0.1", 1, 1));
        parent.add_child(ConfigLine::new("area 0", 1, 2));

        assert_eq!(parent.find_child_with("router-id").unwrap().text, "router-id 10.0.0.1");
        assert!(parent.find_child_with("interface").is_none());
    }

    #[test]
    fn find_all_children_with() {
        let mut parent = ConfigLine::new("router ospf 1", 0, 0);
        parent.add_child(ConfigLine::new("area 0", 1, 1));
        parent.add_child(ConfigLine::new("area 1", 1, 2));
        parent.add_child(ConfigLine::new("router-id 10.0.0.1", 1, 3));

        let areas = parent.find_all_children_with(r"area \d+");
        assert_eq!(areas.len(), 2);
        assert_eq!(areas[0].text, "area 0");
        assert_eq!(areas[1].text, "area 1");
    }

    #[test]
    fn has_child_with() {
        let mut parent = ConfigLine::new("router ospf 1", 0, 0);
        parent.add_child(ConfigLine::new("router-id 10.0.0.1", 1, 1));
        assert!(parent.has_child_with("router-id"));
        assert!(!parent.has_child_with("interface"));
    }

    #[test]
    fn config_block_get_all_lines_flat() {
        let mut root = ConfigLine::new("router ospf 1", 0, 0);
        root.add_child(ConfigLine::new("router-id 10.0.0.1", 1, 1));
        root.add_child(ConfigLine::new("area 0", 1, 2));

        let block = ConfigBlock::new(root);
        let all = block.get_all_lines();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].text, "router ospf 1");
        assert_eq!(all[1].text, "router-id 10.0.0.1");
        assert_eq!(all[2].text, "area 0");
    }

    #[test]
    fn config_block_get_all_lines_nested() {
        let mut root = ConfigLine::new("router ospf 1", 0, 0);
        let mut area = ConfigLine::new("area 0", 1, 1);
        let mut interface = ConfigLine::new("interface GigabitEthernet0/0/0/0", 2, 2);
        interface.add_child(ConfigLine::new("cost 10", 3, 3));
        area.add_child(interface);
        root.add_child(area);

        let block = ConfigBlock::new(root);
        let all = block.get_all_lines();
        assert_eq!(all.len(), 4);
        assert_eq!(all[3].text, "cost 10");
    }

    #[test]
    fn config_block_find_line_with_nested() {
        let mut root = ConfigLine::new("router ospf 1", 0, 0);
        let mut area = ConfigLine::new("area 0", 1, 1);
        area.add_child(ConfigLine::new("interface GigabitEthernet0/0/0/0", 2, 2));
        root.add_child(area);

        let block = ConfigBlock::new(root);
        assert_eq!(block.find_line_with("interface").unwrap().text, "interface GigabitEthernet0/0/0/0");
    }

    #[test]
    fn config_block_re_search_children() {
        let mut root = ConfigLine::new("router ospf 1", 0, 0);
        let mut area0 = ConfigLine::new("area 0", 1, 1);
        area0.add_child(ConfigLine::new("interface GigabitEthernet0/0/0/0", 2, 2));
        root.add_child(area0);
        root.add_child(ConfigLine::new("area 1", 1, 3));

        let block = ConfigBlock::new(root);
        let areas = block.re_search_children(r"area \d+");
        assert_eq!(areas.len(), 2);
        assert_eq!(areas[0].text, "area 0");
        assert_eq!(areas[1].text, "area 1");
    }
}
