//! Python wrappers for the generic config tree. Each wrapper owns a clone of the Rust value, so
//! Python never holds a borrow into another object.

use asterust::config::{cisco, juniper, Config, ConfigBlock, ConfigLine};
use pyo3::prelude::*;

/// A single configuration line with its nested lines.
///
/// `text` is the trimmed line without a trailing `;` or opening `{`. `indent_level` is the
/// indentation depth for Cisco-style configs and the brace depth for Juniper-style ones.
///
/// `pattern` arguments use Rust `regex` syntax rather than Python's `re`, and an invalid pattern
/// never matches.
#[pyclass(name = "ConfigLine", module = "asterust", frozen)]
pub struct PyConfigLine {
    inner: ConfigLine,
}

impl From<&ConfigLine> for PyConfigLine {
    fn from(line: &ConfigLine) -> Self {
        Self { inner: line.clone() }
    }
}

#[pymethods]
impl PyConfigLine {
    /// Returns the line text.
    #[getter]
    fn text(&self) -> &str {
        &self.inner.text
    }

    /// Returns the indentation depth (Cisco) or brace depth (Juniper).
    #[getter]
    fn indent_level(&self) -> usize {
        self.inner.indent_level
    }

    /// Returns the zero-based index of the line in the parsed text.
    #[getter]
    fn line_number(&self) -> usize {
        self.inner.line_number
    }

    /// Returns the direct children of this line.
    #[getter]
    fn children(&self) -> Vec<PyConfigLine> {
        self.inner.children.iter().map(Into::into).collect()
    }

    /// Returns the first direct child whose text matches `pattern`, or `None`.
    fn find_child_with(&self, pattern: &str) -> Option<PyConfigLine> {
        self.inner.find_child_with(pattern).map(Into::into)
    }

    /// Returns all direct children whose text matches `pattern`.
    fn find_all_children_with(&self, pattern: &str) -> Vec<PyConfigLine> {
        self.inner
            .find_all_children_with(pattern)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    /// Returns whether any direct child's text matches `pattern`.
    fn has_child_with(&self, pattern: &str) -> bool {
        self.inner.has_child_with(pattern)
    }

    /// Matches `pattern` against this line's text and returns capture group 1, or `None`.
    fn match_str_group(&self, pattern: &str) -> Option<String> {
        self.inner.match_str_group(pattern)
    }

    /// Matches `pattern` against this line's text and returns capture group 1 as an int, or `None`
    /// if there is no match or the group is not an integer.
    fn match_int_group(&self, pattern: &str) -> Option<i64> {
        self.inner.match_int_group(pattern)
    }

    fn __repr__(&self) -> String {
        format!("ConfigLine({:?})", self.inner.text)
    }
}

/// A top-level configuration block: its root line and everything nested beneath it.
///
/// `pattern` arguments use Rust `regex` syntax rather than Python's `re`, and an invalid pattern
/// never matches.
#[pyclass(name = "ConfigBlock", module = "asterust", frozen)]
pub struct PyConfigBlock {
    inner: ConfigBlock,
}

impl From<&ConfigBlock> for PyConfigBlock {
    fn from(block: &ConfigBlock) -> Self {
        Self { inner: block.clone() }
    }
}

#[pymethods]
impl PyConfigBlock {
    /// Returns the first line of the block, which names it.
    #[getter]
    fn root_line(&self) -> PyConfigLine {
        self.inner.get_root_line().into()
    }

    /// Returns the direct children of the root line.
    #[getter]
    fn direct_children(&self) -> Vec<PyConfigLine> {
        self.inner.get_direct_children().iter().map(Into::into).collect()
    }

    /// Returns every line in the block, depth-first, starting with the root line.
    #[getter]
    fn all_lines(&self) -> Vec<PyConfigLine> {
        self.inner.get_all_lines().into_iter().map(Into::into).collect()
    }

    /// Returns the first direct child of the root line matching `pattern`, or `None`.
    fn find_child_with(&self, pattern: &str) -> Option<PyConfigLine> {
        self.inner.find_child_with(pattern).map(Into::into)
    }

    /// Returns all direct children of the root line matching `pattern`.
    fn find_all_children_with(&self, pattern: &str) -> Vec<PyConfigLine> {
        self.inner
            .find_all_children_with(pattern)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    /// Returns the first line at any depth matching `pattern`, or `None`.
    fn find_line_with(&self, pattern: &str) -> Option<PyConfigLine> {
        self.inner.find_line_with(pattern).map(Into::into)
    }

    /// Returns every line at any depth matching `pattern`.
    fn re_search_children(&self, pattern: &str) -> Vec<PyConfigLine> {
        self.inner
            .re_search_children(pattern)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn __repr__(&self) -> String {
        format!("ConfigBlock({:?})", self.inner.get_root_line().text)
    }
}

/// A parsed configuration: one `ConfigBlock` per top-level line.
///
/// Supports `len()` (the number of blocks). `pattern` arguments use Rust `regex` syntax rather
/// than Python's `re`, and an invalid pattern never matches.
#[pyclass(name = "Config", module = "asterust", frozen)]
pub struct PyConfig {
    pub(crate) inner: Config,
}

#[pymethods]
impl PyConfig {
    /// Returns the top-level blocks in config order.
    #[getter]
    fn blocks(&self) -> Vec<PyConfigBlock> {
        self.inner.blocks.iter().map(Into::into).collect()
    }

    /// Returns all blocks whose root line matches `pattern`.
    fn find_blocks_by_pattern(&self, pattern: &str) -> Vec<PyConfigBlock> {
        self.inner
            .find_blocks_by_pattern(pattern)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn __len__(&self) -> usize {
        self.inner.get_block_count()
    }

    fn __repr__(&self) -> String {
        format!("Config(blocks={})", self.inner.get_block_count())
    }
}

/// Parses Cisco-style (indentation-based) config text into a `Config`.
///
/// ```python
/// from asterust import parse_cisco
///
/// config = parse_cisco(text)
/// for block in config.find_blocks_by_pattern(r"router ospf"):
///     print(block.root_line.text)
/// ```
#[pyfunction]
pub fn parse_cisco(config_text: &str) -> PyConfig {
    PyConfig {
        inner: cisco::parse(config_text),
    }
}

/// Parses Juniper-style (brace-delimited) config text into a `Config`.
///
/// ```python
/// from asterust import parse_juniper
///
/// config = parse_juniper(text)
/// system = config.find_blocks_by_pattern(r"^system$")[0]
/// ```
#[pyfunction]
pub fn parse_juniper(config_text: &str) -> PyConfig {
    PyConfig {
        inner: juniper::parse_braces(config_text),
    }
}
