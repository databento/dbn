use std::{
    io::{self, Read, Seek},
    num::NonZeroU64,
};

use dbn::encode::dbn::MetadataEncoder;
use pyo3::{exceptions::PyTypeError, intern, prelude::*, types::PyBytes};

/// Updates existing fields that have already been written to the given file.
#[pyfunction]
#[pyo3(signature = (file, start, end = None, limit = None))]
pub fn update_encoded_metadata(
    _py: Python<'_>,
    mut file: PyFileLike,
    start: u64,
    end: Option<u64>,
    limit: Option<u64>,
) -> PyResult<()> {
    file.seek(io::SeekFrom::Start(0))?;
    let mut buf = [0; 4];
    file.read_exact(&mut buf)?;
    let version = buf[3];
    Ok(MetadataEncoder::new(file).update_encoded(
        version,
        start,
        end.and_then(NonZeroU64::new),
        limit.and_then(NonZeroU64::new),
    )?)
}

/// A Python object that implements the Python file interface.
pub struct PyFileLike {
    inner: PyObject,
}

impl<'py> FromPyObject<'py> for PyFileLike {
    fn extract_bound(any: &Bound<'py, pyo3::PyAny>) -> PyResult<Self> {
        Python::with_gil(|py| {
            let obj: PyObject = any.extract()?;
            if obj.getattr(py, intern!(py, "read")).is_err() {
                return Err(PyTypeError::new_err(
                    "object is missing a `read()` method".to_owned(),
                ));
            }
            if obj.getattr(py, intern!(py, "write")).is_err() {
                return Err(PyTypeError::new_err(
                    "object is missing a `write()` method".to_owned(),
                ));
            }
            if obj.getattr(py, intern!(py, "seek")).is_err() {
                return Err(PyTypeError::new_err(
                    "object is missing a `seek()` method".to_owned(),
                ));
            }
            Ok(PyFileLike { inner: obj })
        })
    }
}

impl io::Read for PyFileLike {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Python::with_gil(|py| {
            let bytes: Vec<u8> = self
                .inner
                .call_method_bound(py, intern!(py, "read"), (buf.len(),), None)
                .map_err(py_to_rs_io_err)?
                .extract(py)?;
            buf[..bytes.len()].clone_from_slice(&bytes);
            Ok(bytes.len())
        })
    }
}

impl io::Write for PyFileLike {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        Python::with_gil(|py| {
            let bytes = PyBytes::new_bound(py, buf).to_object(py);
            let number_bytes_written = self
                .inner
                .call_method_bound(py, intern!(py, "write"), (bytes,), None)
                .map_err(py_to_rs_io_err)?;

            number_bytes_written.extract(py).map_err(py_to_rs_io_err)
        })
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        Python::with_gil(|py| {
            self.inner
                .call_method_bound(py, intern!(py, "flush"), (), None)
                .map_err(py_to_rs_io_err)?;

            Ok(())
        })
    }
}

impl io::Seek for PyFileLike {
    fn seek(&mut self, pos: io::SeekFrom) -> Result<u64, io::Error> {
        Python::with_gil(|py| {
            let (whence, offset) = match pos {
                io::SeekFrom::Start(i) => (0, i as i64),
                io::SeekFrom::Current(i) => (1, i),
                io::SeekFrom::End(i) => (2, i),
            };

            let new_position = self
                .inner
                .call_method_bound(py, intern!(py, "seek"), (offset, whence), None)
                .map_err(py_to_rs_io_err)?;

            new_position.extract(py).map_err(py_to_rs_io_err)
        })
    }
}

fn py_to_rs_io_err(e: PyErr) -> io::Error {
    Python::with_gil(|py| {
        let e_as_object: PyObject = e.into_py(py);

        match e_as_object.call_method_bound(py, intern!(py, "__str__"), (), None) {
            Ok(repr) => match repr.extract::<String>(py) {
                Ok(s) => io::Error::new(io::ErrorKind::Other, s),
                Err(_e) => io::Error::new(io::ErrorKind::Other, "An unknown error has occurred"),
            },
            Err(_) => io::Error::new(io::ErrorKind::Other, "Err doesn't have __str__"),
        }
    })
}

#[cfg(test)]
pub mod tests {
    use std::{
        io::{Cursor, Seek, Write},
        sync::{Arc, Mutex},
    };

    use super::*;

    #[pyclass]
    #[derive(Default)]
    pub struct MockPyFile {
        buf: Arc<Mutex<Cursor<Vec<u8>>>>,
    }

    #[pymethods]
    impl MockPyFile {
        fn read(&self) {
            unimplemented!();
        }

        fn write(&mut self, bytes: &[u8]) -> usize {
            self.buf.lock().unwrap().write_all(bytes).unwrap();
            bytes.len()
        }

        fn flush(&mut self) {
            self.buf.lock().unwrap().flush().unwrap();
        }

        fn seek(&self, offset: i64, whence: i32) -> u64 {
            self.buf
                .lock()
                .unwrap()
                .seek(match whence {
                    0 => io::SeekFrom::Start(offset as u64),
                    1 => io::SeekFrom::Current(offset),
                    2 => io::SeekFrom::End(offset),
                    _ => unimplemented!("whence value"),
                })
                .unwrap()
        }
    }

    impl MockPyFile {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn inner(&self) -> Arc<Mutex<Cursor<Vec<u8>>>> {
            self.buf.clone()
        }
    }
}
