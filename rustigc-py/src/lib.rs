// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

#![allow(clippy::useless_conversion)]

use ::rustigc;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList, PyTuple};
use rustigc::{FlightDetection, GeoJson};

/// Python binding interface for IGC log
#[pyclass(name = "RustLog")]
struct PyLog {
    inner: rustigc::Log,
}

/// A layer of a flight, kept alive on this side so it can be drawn later.
///
/// Python reads a layer's scalars from `json`, and hands the handle itself back to [`PyLog::export`].
/// Neither `Flight` nor `ScoringResult` can be rebuilt from that JSON — `ScoringResult` carries a
/// `&'static str` — and re-deriving one would mean detecting or scoring a second time.
macro_rules! layer {
    ($name:ident, $py_name:literal, $inner:ty, $what:literal) => {
        #[pyclass(name = $py_name)]
        struct $name {
            inner: $inner,
        }

        #[pymethods]
        impl $name {
            /// The layer as a JSON dump, the shape its Python wrapper reads its scalars from
            fn json(&self) -> PyResult<String> {
                serde_json::to_string(&self.inner).map_err(|e| {
                    PyValueError::new_err(format!("Failed to serialize {}: {e}", $what))
                })
            }
        }
    };
}

layer!(PyFlight, "RustFlight", rustigc::Flight, "flight");
layer!(PyScore, "RustScore", rustigc::ScoringResult, "score");

/// One borrowed layer handle, held while the slice of trait objects pointing into it is built.
enum Layer<'py> {
    Flight(PyRef<'py, PyFlight>),
    Score(PyRef<'py, PyScore>),
}

impl<'py> Layer<'py> {
    fn borrow(item: &Bound<'py, PyAny>) -> PyResult<Self> {
        if let Ok(flight) = item.extract::<PyRef<'py, PyFlight>>() {
            return Ok(Self::Flight(flight));
        }
        if let Ok(score) = item.extract::<PyRef<'py, PyScore>>() {
            return Ok(Self::Score(score));
        }

        Err(PyTypeError::new_err(format!(
            "cannot draw a {}",
            item.get_type()
        )))
    }

    fn as_layer(&self) -> &dyn GeoJson {
        match self {
            Self::Flight(flight) => &flight.inner,
            Self::Score(score) => &score.inner,
        }
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

        Ok(PyLog { inner })
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

    /// Replace the track with `data`, laid out for `FIX_DTYPE`. The fix count may change.
    fn set_track_bytes(&mut self, py: Python<'_>, data: &[u8]) -> PyResult<()> {
        let stride = std::mem::size_of::<rustigc::Fix>();
        if !data.len().is_multiple_of(stride) {
            return Err(PyValueError::new_err(format!(
                "track of {} bytes is not a whole number of {stride}-byte fixes",
                data.len()
            )));
        }

        let count = data.len() / stride;
        let track: Vec<rustigc::Fix> = py.allow_threads(|| {
            let mut track = Vec::<rustigc::Fix>::with_capacity(count);
            // SAFETY: reinterpreting `data` as `Fix`es is safe because:
            // - Fix is repr(C) and still a guraenteed layout
            // - the length check above makes `count * stride` exactly `data.len()`
            // - `with_capacity` gives an allocation aligned for Fix, large enough, and distinct
            //   from `data`, so the copy cannot overlap
            unsafe {
                std::ptr::copy_nonoverlapping(
                    data.as_ptr(),
                    track.as_mut_ptr() as *mut u8,
                    data.len(),
                );
                track.set_len(count);
            }
            track
        });

        // Make sure the timestamps are stricly increasing
        if let Some(w) = track.windows(2).find(|w| w[1].timestamp <= w[0].timestamp) {
            return Err(PyValueError::new_err(format!(
                "timestamps must be strictly increasing, got {} then {}",
                w[0].timestamp, w[1].timestamp
            )));
        }

        self.inner.track = track;
        Ok(())
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

    /// Instant this log's fix timestamps count from, or `None` without a usable `HFDTE` header.
    ///
    /// UTC midnight of the flight's date, in the zone the track starts in, as RFC 9557 —
    /// `2022-08-05T01:00:00+01:00[Europe/London]`. That is what a `Zoned` prints, offset and zone
    /// name together, so the other side can rebuild the zone itself rather than be handed pieces.
    fn datetime(&self) -> Option<String> {
        self.inner.datetime().map(|origin| origin.to_string())
    }

    /// Detect the flight sections, one handle each
    fn flights(&self, py: Python<'_>) -> Vec<PyFlight> {
        let flights = py.allow_threads(|| self.inner.track.flights());

        flights
            .into_iter()
            .map(|inner| PyFlight { inner })
            .collect()
    }

    /// Score the fixes in `[start, stop]` against `league`
    fn score(
        &self,
        py: Python<'_>,
        league: &str,
        start: usize,
        stop: usize,
    ) -> Option<PyScore> {
        // FIXME: `None` covers every way this can fail: unknown league, bad window, nothing scored.
        // Something to fixme in the core
        let result = py.allow_threads(|| self.inner.score(league, start, stop));

        result.map(|inner| PyScore { inner })
    }

    /// Everything the log describes about itself, as one GeoJSON string
    fn describe(&self, py: Python<'_>, league: &str) -> PyResult<String> {
        py.allow_threads(|| serde_json::to_string(&self.inner.describe(league)))
            .map_err(|e| {
                PyValueError::new_err(format!("Failed to serialize geojson: {e}"))
            })
    }

    /// The log, its time reference and each of `layers`, in the order given, as one GeoJSON string.
    ///
    /// `track` draws the flown line. Fix indices are taken on trust: nothing checks that a layer was
    /// detected or scored in the track this log holds now, which `set_track_bytes` can replace.
    #[pyo3(signature = (layers, track = true))]
    fn export(
        &self,
        py: Python<'_>,
        layers: Vec<Bound<'_, PyAny>>,
        track: bool,
    ) -> PyResult<String> {
        // Borrow every handle first: the slice below holds references into them.
        let held: Vec<Layer<'_>> =
            layers.iter().map(Layer::borrow).collect::<PyResult<_>>()?;
        let layers: Vec<&dyn GeoJson> = held.iter().map(Layer::as_layer).collect();

        let line = if track {
            rustigc::TrackLine::Draw
        } else {
            rustigc::TrackLine::Skip
        };

        py.allow_threads(|| serde_json::to_string(&self.inner.export_with(&layers, line)))
            .map_err(|e| {
                PyValueError::new_err(format!("Failed to serialize geojson: {e}"))
            })
    }

    fn __repr__(&self) -> String {
        let pilot = self.get_header("PLT").map(|(text, _)| text);
        format!("Log(fixes={}, pilot={:?})", self.inner.track.len(), pilot)
    }
}

/// Every league `score` accepts
#[pyfunction]
fn league_names() -> Vec<&'static str> {
    rustigc::league_names().collect()
}

/// Python minimal bindings for rustigc parsing library
#[pymodule]
#[pyo3(name = "_bindings")]
fn rustigc_py_bindings(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<PyLog>()?;
    m.add_class::<PyFlight>()?;
    m.add_class::<PyScore>()?;
    m.add_function(wrap_pyfunction!(league_names, m)?)?;

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
