#![allow(clippy::useless_conversion)]

use ::rustigc;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList, PyTuple};
use std::sync::OnceLock;

/// Python binding interface for IGC log
#[pyclass(name = "RustLog")]
struct PyLog {
    inner: rustigc::Log,
    analysis: OnceLock<rustigc::Analysis>,
}

impl PyLog {
    fn analysis(&self) -> &rustigc::Analysis {
        self.analysis
            .get_or_init(|| rustigc::Analysis::new(&self.inner.track))
    }
}

#[pymethods]
impl PyLog {
    /// Parse an IGC file content
    #[staticmethod]
    fn from_bytes(py: Python<'_>, content: &[u8]) -> PyResult<Self> {
        let inner = py
            .allow_threads(|| rustigc::Log::new(content))
            .map_err(|e| {
                PyValueError::new_err(format!("Failed to parse IGC file: {e}"))
            })?;

        Ok(PyLog {
            inner,
            analysis: OnceLock::new(),
        })
    }

    /// Get track as raw bytes, laid out for `FIX_DTYPE` (32 bytes per fix with padding).
    #[getter]
    fn track_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        // SAFETY: Creating a byte slice view of Vec<Fix> is safe because:
        // - Fix is repr(C) with guaranteed layout
        // - Pointer is valid for the duration of this borrow
        // - PyBytes::new_bound immediately copies data (no aliasing)
        let bytes = unsafe {
            std::slice::from_raw_parts(
                self.inner.track.as_ptr() as *const u8,
                self.inner.track.len() * std::mem::size_of::<rustigc::Fix>(),
            )
        };
        PyBytes::new_bound(py, bytes)
    }

    /// Get header value and origin by key (e.g., "PLT", "GTY", "DTE")
    /// Returns tuple of (text, origin), origin being "Flight Recorder", "Observer", "Pilot" or
    /// "Unknown"
    fn get_header(&self, key: &str) -> Option<(String, String)> {
        if key.len() != 3 {
            return None;
        }
        self.inner
            .headers
            .get(key)
            .map(|data| (data.text.clone(), data.origin.as_str().to_string()))
    }

    /// Forward flight analysis manual trigger
    fn analyze(&self) {
        self.analysis();
    }

    /// Takeoff fix index
    #[getter]
    fn takeoff(&self) -> Option<usize> {
        self.analysis().flight().map(|(t, _)| t)
    }

    /// Landing fix index
    #[getter]
    fn landing(&self) -> Option<usize> {
        self.analysis().flight().map(|(_, l)| l)
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
                PyTuple::new_bound(py, ["timestamp", "u4"]),
                PyTuple::new_bound(py, ["_pad", "u4"]),
                PyTuple::new_bound(py, ["latitude", "f8"]),
                PyTuple::new_bound(py, ["longitude", "f8"]),
                PyTuple::new_bound(py, ["baro_altitude", "i4"]),
                PyTuple::new_bound(py, ["gnss_altitude", "i4"]),
            ],
        );

        let dtype = numpy.getattr("dtype")?.call1((dtype_spec,))?;
        m.add("FIX_DTYPE", dtype)?;

        Ok::<(), PyErr>(())
    })?;

    Ok(())
}
