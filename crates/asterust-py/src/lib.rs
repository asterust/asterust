mod config;
mod iosxr;

use pyo3::prelude::*;

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<config::PyConfig>()?;
    m.add_class::<config::PyConfigBlock>()?;
    m.add_class::<config::PyConfigLine>()?;
    m.add_function(wrap_pyfunction!(config::parse_cisco, m)?)?;
    m.add_function(wrap_pyfunction!(config::parse_juniper, m)?)?;

    m.add_class::<iosxr::PyIosXrDevice>()?;
    m.add_class::<iosxr::PyInterface>()?;
    m.add_class::<iosxr::PyOspfArea>()?;
    m.add_class::<iosxr::PyOspfProcess>()?;
    m.add_class::<iosxr::PyBgpNeighbor>()?;
    m.add_class::<iosxr::PyBgpProcess>()?;
    m.add_class::<iosxr::PyIsisInstance>()?;
    Ok(())
}
