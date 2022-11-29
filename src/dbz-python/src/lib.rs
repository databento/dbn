use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

/// A Python module wrapping dbz-lib functions
#[pymodule] // The name of the function must match `lib.name` in `Cargo.toml`
fn dbz_python(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // all functions exposed to Python need to be added here
    m.add_wrapped(wrap_pyfunction!(dbz_lib::python::decode_metadata))?;
    m.add_wrapped(wrap_pyfunction!(dbz_lib::python::encode_metadata))?;
    m.add_wrapped(wrap_pyfunction!(dbz_lib::python::update_encoded_metadata))?;
    m.add_wrapped(wrap_pyfunction!(dbz_lib::python::write_dbz_file))?;
    Ok(())
}
