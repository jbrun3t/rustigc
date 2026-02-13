#![allow(clippy::useless_conversion)]

use ::rustigc;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Python wrapper for Fix (position fix)
#[pyclass(name = "Fix")]
#[derive(Clone)]
struct PyFix {
    inner: rustigc::Fix,
}

#[pymethods]
impl PyFix {
    /// Get the timestamp in seconds since midnight (0-86399)
    #[getter]
    fn timestamp(&self) -> u32 {
        self.inner.timestamp
    }

    /// Get latitude in decimal degrees
    #[getter]
    fn latitude(&self) -> f64 {
        self.inner.lat
    }

    /// Get longitude in decimal degrees
    #[getter]
    fn longitude(&self) -> f64 {
        self.inner.lon
    }

    /// Get pressure altitude in meters
    #[getter]
    fn baro_altitude(&self) -> i32 {
        self.inner.baro_alt
    }

    /// Get GNSS altitude in meters
    #[getter]
    fn gnss_altitude(&self) -> i32 {
        self.inner.gnss_alt
    }

    fn __repr__(&self) -> String {
        format!(
            "Fix(lat={:.6}, lon={:.6}, alt={}m)",
            self.latitude(),
            self.longitude(),
            self.gnss_altitude()
        )
    }
}

/// Python wrapper for Track
#[pyclass(name = "Log")]
struct PyLog {
    inner: rustigc::Log,
    flight: Option<(usize, usize)>,
}

#[pymethods]
impl PyLog {
    /// Parse an IGC file from bytes
    #[staticmethod]
    fn from_bytes(content: &[u8]) -> PyResult<Self> {
        let inner = rustigc::Log::new(content).map_err(|e| {
            PyValueError::new_err(format!("Failed to parse IGC file: {e}"))
        })?;

        let data = rustigc::FRawData::new(&inner);
        let flight = data.phases();

        Ok(PyLog {
            inner,
            flight,
        })
    }

    /// Get the number of fixes
    fn __len__(&self) -> usize {
        self.inner.track.len()
    }

    /// Get a specific fix by index
    fn __getitem__(&self, idx: isize) -> PyResult<PyFix> {
        let len = self.inner.track.len() as isize;
        let idx = if idx < 0 { len + idx } else { idx };

        if idx < 0 || idx >= len {
            return Err(PyValueError::new_err("Index out of range"));
        }

        Ok(PyFix {
            inner: self.inner.track[idx as usize].clone(),
        })
    }

    /// Get all fixes as a list
    fn fixes(&self) -> Vec<PyFix> {
        self.inner
            .track
            .iter()
            .map(|fix| PyFix { inner: fix.clone() })
            .collect()
    }

    fn get_header(&self, key: &str) -> Option<String> {
        self.inner.headers.get(key).map(|data| data.text.clone())
    }

    /// Get the pilot name
    fn pilot_name(&self) -> Option<String> {
        self.get_header("PLT")
    }

    /// Get the glider type
    fn glider_type(&self) -> Option<String> {
        self.get_header("GTY")
    }

    /// Get the flight date (convenience method)
    /// Returns (year, month, day) tuple or None
    fn date(&self) -> Option<String> {
        self.get_header("DTE")
    }

    /// Detect and return the takeoff fix (or None if not detected)
    #[getter]
    fn takeoff(&self) -> Option<usize> {
        self.flight.map(|f| f.0)
    }

    /// Detect and return the landing fix (or None if not detected)
    #[getter]
    fn landing(&self) -> Option<usize> {
        self.flight.map(|f| f.1)
    }

    fn __repr__(&self) -> String {
        format!(
            "Log(fixes={}, pilot={:?})",
            self.__len__(),
            self.pilot_name()
        )
    }
}

/// Python module for rustigc
#[pymodule]
fn rustigcpy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<PyLog>()?;
    m.add_class::<PyFix>()?;
    Ok(())
}
