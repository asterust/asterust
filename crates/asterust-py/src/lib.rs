//! Python bindings for `asterust` and its platform crates, built as one extension module.
//!
//! Everything is compiled into `asterust._core`; the `python/asterust` package re-exports the
//! generic config types at the top level and the platform views from submodules such as
//! `asterust.iosxr`.

#[macro_use]
mod repr;

mod config;
mod iosxe;
mod iosxr;

use pyo3::prelude::*;

/// Native extension module behind the `asterust` package. Import `asterust` or a platform
/// submodule such as `asterust.iosxr` rather than this module.
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<config::PyConfig>()?;
    m.add_class::<config::PyConfigBlock>()?;
    m.add_class::<config::PyConfigLine>()?;
    m.add_function(wrap_pyfunction!(config::parse_cisco, m)?)?;
    m.add_function(wrap_pyfunction!(config::parse_juniper, m)?)?;

    m.add_class::<iosxe::PyIosXeDevice>()?;
    m.add_class::<iosxe::PyIosXeInterface>()?;
    m.add_class::<iosxe::PyIosXeOspfNetwork>()?;
    m.add_class::<iosxe::PyIosXeOspfProcess>()?;
    m.add_class::<iosxe::PyIosXeBgpNeighbor>()?;
    m.add_class::<iosxe::PyIosXeBgpProcess>()?;
    m.add_class::<iosxe::PyIosXeIsisInstance>()?;
    m.add_class::<iosxe::PyIosXeVrfAddressFamily>()?;
    m.add_class::<iosxe::PyIosXeVrf>()?;

    m.add_class::<iosxr::PyIosXrDevice>()?;
    m.add_class::<iosxr::PyInterface>()?;
    m.add_class::<iosxr::PyOspfArea>()?;
    m.add_class::<iosxr::PyOspfProcess>()?;
    m.add_class::<iosxr::PyBgpNeighbor>()?;
    m.add_class::<iosxr::PyBgpProcess>()?;
    m.add_class::<iosxr::PyIsisInstance>()?;
    m.add_class::<iosxr::PyBgpTimers>()?;
    m.add_class::<iosxr::PyBgpNeighborAddressFamily>()?;
    m.add_class::<iosxr::PyBgpAfGroup>()?;
    m.add_class::<iosxr::PyBgpNeighborGroup>()?;
    m.add_class::<iosxr::PyBgpSessionGroup>()?;
    m.add_class::<iosxr::PyBgpAddressFamily>()?;
    m.add_class::<iosxr::PyBgpVrf>()?;
    m.add_class::<iosxr::PyVrfAddressFamily>()?;
    m.add_class::<iosxr::PyVrf>()?;
    Ok(())
}
