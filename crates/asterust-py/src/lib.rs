#[macro_use]
mod repr;

mod config;
mod iosxe;
mod iosxr;

use pyo3::prelude::*;

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

    m.add_class::<iosxr::PyIosXrDevice>()?;
    m.add_class::<iosxr::PyInterface>()?;
    m.add_class::<iosxr::PyOspfArea>()?;
    m.add_class::<iosxr::PyOspfProcess>()?;
    m.add_class::<iosxr::PyBgpNeighbor>()?;
    m.add_class::<iosxr::PyBgpProcess>()?;
    m.add_class::<iosxr::PyIsisInstance>()?;
    Ok(())
}
