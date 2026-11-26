//! Implements a shared `__repr__` for the data classes: their `Debug` output with the Rust-side
//! `Py` prefix dropped.

macro_rules! impl_repr {
    ($($ty:ty),*) => {$(
        #[pymethods]
        impl $ty {
            fn __repr__(&self) -> String {
                let debug = format!("{self:?}");
                debug.strip_prefix("Py").map(str::to_owned).unwrap_or(debug)
            }
        }
    )*};
}
