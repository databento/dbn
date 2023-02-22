//! Python bindings for the [`dbn`] crate using [`pyo3`].
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

/// A Python module wrapping dbn functions
#[pymodule] // The name of the function must match `lib.name` in `Cargo.toml`
fn databento_dbn(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // all functions exposed to Python need to be added here
    m.add_wrapped(wrap_pyfunction!(dbn::python::decode_metadata))?;
    m.add_wrapped(wrap_pyfunction!(dbn::python::encode_metadata))?;
    m.add_wrapped(wrap_pyfunction!(dbn::python::update_encoded_metadata))?;
    m.add_wrapped(wrap_pyfunction!(dbn::python::write_dbn_file))?;
    Ok(())
}
