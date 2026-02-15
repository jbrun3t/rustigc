#![allow(clippy::useless_conversion)]

use ::rustigc;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList, PyTuple};
use std::cell::RefCell;

/// Python binding interface for IGC log
#[pyclass(name = "RustLog")]
struct PyLog {
    inner: rustigc::Log,
    data: RefCell<Option<rustigc::FRawData>>,
    phases: RefCell<Option<(usize, usize)>>,
}

#[pymethods]
impl PyLog {
    /// Parse an IGC file content
    #[staticmethod]
    fn from_bytes(content: &[u8]) -> PyResult<Self> {
        let inner = rustigc::Log::new(content).map_err(|e| {
            PyValueError::new_err(format!("Failed to parse IGC file: {e}"))
        })?;

        Ok(PyLog {
            inner,
            data: RefCell::new(None),
            phases: RefCell::new(None),
        })
    }

    /// Get track as raw bytes for zero-copy numpy access (32 bytes per fix with padding)
    #[getter]
    fn track_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        // SAFETY: Creating a byte slice view of Vec<Fix> is safe because:
        // - Fix is repr(C) with guaranteed layout
        // - Pointer is valid for the duration of this borrow
        // - PyBytes::new_bound immediately copies data (no aliasing)
        // - Length calculation is correct: track.len() * size_of::<Fix>()
        let bytes = unsafe {
            std::slice::from_raw_parts(
                self.inner.track.as_ptr() as *const u8,
                self.inner.track.len() * std::mem::size_of::<rustigc::Fix>(),
            )
        };
        PyBytes::new_bound(py, bytes)
    }

    /// Get header value and origin by key (e.g., "PLT", "GTY", "DTE")
    /// Returns tuple of (text, origin) where origin is "FlightRecorder", "Observer", or "Pilot"
    fn get_header(&self, key: &str) -> Option<(String, String)> {
        if key.len() != 3 {
            return None;
        }
        let key_bytes: [u8; 3] = key.as_bytes().try_into().ok()?;
        self.inner
            .headers
            .get(&key_bytes)
            .map(|data| (data.text.clone(), data.origin.as_str().to_string()))
    }

    /// Flight analysis manual trigger
    fn analyze(&self) {
        if self.data.borrow().is_none() {
            let data = rustigc::FRawData::new(&self.inner);
            let phases = data.phases();
            *self.data.borrow_mut() = Some(data);
            *self.phases.borrow_mut() = phases;
        }
    }

    /// Takeoff fix index
    #[getter]
    fn takeoff(&self) -> Option<usize> {
        self.analyze();
        self.phases.borrow().map(|f| f.0)
    }

    /// Landing fix index
    #[getter]
    fn landing(&self) -> Option<usize> {
        self.analyze();
        self.phases.borrow().map(|f| f.1)
    }

    fn __repr__(&self) -> String {
        let pilot = self.get_header("PLT").map(|(text, _)| text);
        format!("Log(fixes={}, pilot={:?})", self.inner.track.len(), pilot)
    }
}

/// Python minimal bindings for rustigc parsing library
#[pymodule]
#[pyo3(name = "_bindings")]
fn rustigc_py_bindings(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<PyLog>()?;

    // Export FIX_DTYPE as numpy dtype
    Python::with_gil(|py| {
        let numpy = py.import_bound("numpy")?;

        // Create dtype matching Fix layout:
        // timestamp first, then padding, then coordinates/altitudes
        let dtype_spec = PyList::new_bound(
            py,
            &[
                PyTuple::new_bound(py, &["timestamp", "u4"]),
                PyTuple::new_bound(py, &["_pad", "u4"]),
                PyTuple::new_bound(py, &["latitude", "f8"]),
                PyTuple::new_bound(py, &["longitude", "f8"]),
                PyTuple::new_bound(py, &["baro_altitude", "i4"]),
                PyTuple::new_bound(py, &["gnss_altitude", "i4"]),
            ],
        );

        let dtype = numpy.getattr("dtype")?.call1((dtype_spec,))?;
        m.add("FIX_DTYPE", dtype)?;

        Ok::<(), PyErr>(())
    })?;

    Ok(())
}
