//! Python wrappers for the generic config tree. Each wrapper owns a clone of the Rust value, so
//! Python never holds a borrow into another object.

use asterust::config::{cisco, juniper, Config, ConfigBlock, ConfigLine};
use pyo3::prelude::*;

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
    #[getter]
    fn text(&self) -> &str {
        &self.inner.text
    }

    #[getter]
    fn indent_level(&self) -> usize {
        self.inner.indent_level
    }

    #[getter]
    fn line_number(&self) -> usize {
        self.inner.line_number
    }

    #[getter]
    fn children(&self) -> Vec<PyConfigLine> {
        self.inner.children.iter().map(Into::into).collect()
    }

    fn find_child_with(&self, pattern: &str) -> Option<PyConfigLine> {
        self.inner.find_child_with(pattern).map(Into::into)
    }

    fn find_all_children_with(&self, pattern: &str) -> Vec<PyConfigLine> {
        self.inner
            .find_all_children_with(pattern)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn has_child_with(&self, pattern: &str) -> bool {
        self.inner.has_child_with(pattern)
    }

    fn match_str_group(&self, pattern: &str) -> Option<String> {
        self.inner.match_str_group(pattern)
    }

    fn match_int_group(&self, pattern: &str) -> Option<i64> {
        self.inner.match_int_group(pattern)
    }

    fn __repr__(&self) -> String {
        format!("ConfigLine({:?})", self.inner.text)
    }
}

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
    #[getter]
    fn root_line(&self) -> PyConfigLine {
        self.inner.get_root_line().into()
    }

    #[getter]
    fn direct_children(&self) -> Vec<PyConfigLine> {
        self.inner.get_direct_children().iter().map(Into::into).collect()
    }

    #[getter]
    fn all_lines(&self) -> Vec<PyConfigLine> {
        self.inner.get_all_lines().into_iter().map(Into::into).collect()
    }

    fn find_child_with(&self, pattern: &str) -> Option<PyConfigLine> {
        self.inner.find_child_with(pattern).map(Into::into)
    }

    fn find_all_children_with(&self, pattern: &str) -> Vec<PyConfigLine> {
        self.inner
            .find_all_children_with(pattern)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn find_line_with(&self, pattern: &str) -> Option<PyConfigLine> {
        self.inner.find_line_with(pattern).map(Into::into)
    }

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

#[pyclass(name = "Config", module = "asterust", frozen)]
pub struct PyConfig {
    pub(crate) inner: Config,
}

#[pymethods]
impl PyConfig {
    #[getter]
    fn blocks(&self) -> Vec<PyConfigBlock> {
        self.inner.blocks.iter().map(Into::into).collect()
    }

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

/// Parse Cisco-style (indentation-based) config text.
#[pyfunction]
pub fn parse_cisco(config_text: &str) -> PyConfig {
    PyConfig {
        inner: cisco::parse(config_text),
    }
}

/// Parse Juniper-style (brace-delimited) config text.
#[pyfunction]
pub fn parse_juniper(config_text: &str) -> PyConfig {
    PyConfig {
        inner: juniper::parse_braces(config_text),
    }
}
