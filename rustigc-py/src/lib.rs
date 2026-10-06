// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

#![allow(clippy::useless_conversion)]

use std::sync::Mutex;

use ::rustigc;
use numpy::{PyReadonlyArray2, PyUntypedArrayMethods};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList, PyTuple};
use rustigc::{FlightDetection, Ruleset};

/// A parsed IGC log, as Rust holds it.
///
/// The raw binding behind `rustigc_py.Log`, which is the interface to use.
#[pyclass(name = "RustLog")]
struct PyLog {
    inner: rustigc::Log,
}

#[pymethods]
impl PyLog {
    /// Parse IGC content, raising ValueError when it is not usable.
    #[staticmethod]
    #[pyo3(text_signature = "(content)")]
    fn from_bytes(py: Python<'_>, content: &[u8]) -> PyResult<Self> {
        let inner = py.detach(|| rustigc::Log::new(content)).map_err(|e| {
            PyValueError::new_err(format!("Failed to parse IGC file: {e}"))
        })?;

        Ok(PyLog { inner })
    }

    /// The track as raw bytes, laid out for `FIX_DTYPE`: 32 per fix, padding included.
    #[getter]
    fn track_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        // SAFETY: `Fix` is `repr(C)`, the pointer is valid for this borrow, and `PyBytes::new`
        // copies the bytes before it ends.
        let bytes = unsafe {
            std::slice::from_raw_parts(
                self.inner.track.as_ptr() as *const u8,
                self.inner.track.len() * std::mem::size_of::<rustigc::Fix>(),
            )
        };
        PyBytes::new(py, bytes)
    }

    /// A new log with `data` as its track, laid out for `FIX_DTYPE`, and this log's recorder,
    /// headers and task. `data` may hold any number of fixes.
    ///
    /// Raises ValueError unless `data` is a whole number of fixes with strictly increasing
    /// timestamps.
    #[pyo3(text_signature = "($self, data)")]
    fn with_track_bytes(&self, py: Python<'_>, data: &[u8]) -> PyResult<Self> {
        let stride = std::mem::size_of::<rustigc::Fix>();
        if !data.len().is_multiple_of(stride) {
            return Err(PyValueError::new_err(format!(
                "track of {} bytes is not a whole number of {stride}-byte fixes",
                data.len()
            )));
        }

        let count = data.len() / stride;
        let track: Vec<rustigc::Fix> = py.detach(|| {
            let mut track = Vec::<rustigc::Fix>::with_capacity(count);
            // SAFETY: `Fix` is `repr(C)`; `count * stride` is exactly `data.len()`; the new
            // allocation is aligned for `Fix`, large enough, and distinct from `data`.
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

        if let Some(w) = track.windows(2).find(|w| w[1].timestamp <= w[0].timestamp) {
            return Err(PyValueError::new_err(format!(
                "timestamps must be strictly increasing, got {} then {}",
                w[0].timestamp, w[1].timestamp
            )));
        }

        // Field by field, so the old track is not copied.
        Ok(PyLog {
            inner: rustigc::Log {
                recorder: self.inner.recorder.clone(),
                headers: self.inner.headers.clone(),
                track,
                task: self.inner.task.clone(),
            },
        })
    }

    /// One header as `(text, origin)`, or `None` when the log has no such key.
    ///
    /// `key` is a 3-letter code: `"PLT"`, `"GTY"`, `"DTE"`, ... `origin` is who entered it:
    /// `"Flight Recorder"`, `"Observer"`, `"Pilot"` or `"Unknown"`.
    #[pyo3(text_signature = "($self, key)")]
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
    /// UTC midnight of the flight's date, ISO 8601, e.g. `2022-08-05T00:00:00.000Z`.
    #[pyo3(text_signature = "($self)")]
    fn datetime(&self) -> Option<String> {
        self.inner.datetime()
    }

    /// Offset from UTC to local time in hours, as the `TZN` header declares it, or `None`.
    #[pyo3(text_signature = "($self)")]
    fn tzn(&self) -> Option<f64> {
        self.inner.tzn()
    }

    /// The detected flight sections, as a JSON array, empty when none is found.
    #[pyo3(text_signature = "($self)")]
    fn flights(&self, py: Python<'_>) -> PyResult<String> {
        let flights = py.detach(|| self.inner.track.flights());

        serde_json::to_string(&flights).map_err(|e| {
            PyValueError::new_err(format!("Failed to serialize flights: {e}"))
        })
    }

    /// Score the fixes in `[start, stop]` against every rule of `ruleset`, reporting the best.
    ///
    /// `None` when no rule scores the window. Raises ValueError for an unknown ruleset, or a
    /// window this log's track does not hold.
    #[pyo3(text_signature = "($self, ruleset, start, stop)")]
    fn score(
        &self,
        py: Python<'_>,
        ruleset: &str,
        start: usize,
        stop: usize,
    ) -> PyResult<Option<String>> {
        let ruleset = named(ruleset)?;
        let result = py
            .detach(|| self.inner.score(ruleset, start, stop))
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        result
            .map(|scored| serde_json::to_string(&scored))
            .transpose()
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize score: {e}")))
    }

    /// GeoJSON of the track, its longest detected flight and that flight's score under
    /// `ruleset`. Raises ValueError for an unknown ruleset.
    #[pyo3(text_signature = "($self, ruleset)")]
    fn describe(&self, py: Python<'_>, ruleset: &str) -> PyResult<String> {
        let ruleset = named(ruleset)?;
        py.detach(|| {
            let collection = self.inner.describe(ruleset).map_err(|e| e.to_string())?;

            serde_json::to_string(&collection)
                .map_err(|e| format!("Failed to serialize geojson: {e}"))
        })
        .map_err(PyValueError::new_err)
    }

    /// GeoJSON of the log, drawing `flight` and `score` when given and the track line when
    /// `track`. `flight` and `score` are the JSON their Python wrappers hold.
    ///
    /// Raises ValueError for JSON that is not a flight or a score, or a fix index past the end of
    /// the track. Indices are not checked against the track they came from.
    #[pyo3(
        signature = (flight = None, score = None, track = true),
        text_signature = "($self, flight=None, score=None, track=True)"
    )]
    fn export(
        &self,
        py: Python<'_>,
        flight: Option<&str>,
        score: Option<&str>,
        track: bool,
    ) -> PyResult<String> {
        let flight: Option<rustigc::Flight> = flight
            .map(serde_json::from_str)
            .transpose()
            .map_err(|e| PyValueError::new_err(format!("Not a flight: {e}")))?;

        let score: Option<rustigc::ScoringResult> = score
            .map(serde_json::from_str)
            .transpose()
            .map_err(|e| PyValueError::new_err(format!("Not a score: {e}")))?;

        let line = if track {
            rustigc::TrackLine::Draw
        } else {
            rustigc::TrackLine::Skip
        };

        py.detach(|| {
            let collection = self
                .inner
                .export(flight, score.as_ref(), line)
                .map_err(|e| e.to_string())?;

            serde_json::to_string(&collection)
                .map_err(|e| format!("Failed to serialize geojson: {e}"))
        })
        .map_err(PyValueError::new_err)
    }

    fn __repr__(&self) -> String {
        let pilot = self.get_header("PLT").map(|(text, _)| text);
        format!("Log(fixes={}, pilot={:?})", self.inner.track.len(), pilot)
    }
}

/// A scoring window over a table of coordinates, needing no log.
///
/// The raw binding behind `rustigc_py.Scorer`, which is the interface to use.
#[pyclass(name = "RustScorer")]
struct PyScorer {
    inner: Mutex<rustigc::Scorer>,
}

#[pymethods]
impl PyScorer {
    /// Prepare an `(N, 2)` float64 array of `[latitude, longitude]`, in degrees and flight order.
    ///
    /// Raises ValueError unless the array is C-contiguous, two columns wide, and holds at least
    /// two points whose coordinates are all in range.
    #[new]
    #[pyo3(text_signature = "(points)")]
    fn new(points: PyReadonlyArray2<f64>) -> PyResult<Self> {
        // Checked on the shape: an (N, 3) array could still split into pairs.
        let &[_, 2] = points.shape() else {
            return Err(PyValueError::new_err(format!(
                "points must be an (N, 2) array of [latitude, longitude], got {:?}",
                points.shape()
            )));
        };

        let flat = points.as_slice().map_err(|_| {
            PyValueError::new_err(
                "points must be C-contiguous; try numpy.ascontiguousarray",
            )
        })?;

        // One copy, into the `Scorer`.
        let (table, _) = flat.as_chunks::<2>();

        let inner = rustigc::Scorer::from_vec(table.to_vec())
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(Self {
            inner: Mutex::new(inner),
        })
    }

    /// Score the table against every rule of `ruleset`, reporting the best. Every fix of the
    /// result is an index into the table.
    ///
    /// `None` when no rule scores the table. Raises ValueError for an unknown ruleset.
    #[pyo3(text_signature = "($self, ruleset)")]
    fn score(&self, py: Python<'_>, ruleset: &str) -> PyResult<Option<String>> {
        let ruleset = named(ruleset)?;
        py.detach(|| {
            let scorer = self.inner.lock().unwrap();
            scorer.solve(ruleset)
        })
        .map(|scored| serde_json::to_string(&scored))
        .transpose()
        .map_err(|e| PyValueError::new_err(format!("Failed to serialize score: {e}")))
    }
}

/// The embedded ruleset `name` refers to; ValueError when there is none.
fn named(name: &str) -> PyResult<&'static Ruleset> {
    Ruleset::named(name).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Every ruleset name `score` and `describe` accept.
#[pyfunction]
#[pyo3(text_signature = "()")]
fn ruleset_names() -> Vec<&'static str> {
    rustigc::ruleset_names().collect()
}

/// Raw bindings to the rustigc library.
///
/// Not the interface to use: `rustigc_py` wraps this with numpy tracks and Python objects. It holds
/// `RustLog`, `RustScorer`, `ruleset_names` and `FIX_DTYPE`. Flights and scores cross as JSON.
#[pymodule]
#[pyo3(name = "_bindings")]
fn rustigc_py_bindings(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<PyLog>()?;
    m.add_class::<PyScorer>()?;
    m.add_function(wrap_pyfunction!(ruleset_names, m)?)?;

    Python::attach(|py| {
        let numpy = py.import("numpy")?;

        // `rustigc::Fix`'s `repr(C)` layout, padding included.
        let dtype_spec = PyList::new(
            py,
            &[
                PyTuple::new(py, ["timestamp", "u4"])?,
                PyTuple::new(py, ["_pad", "u4"])?,
                PyTuple::new(py, ["latitude", "f8"])?,
                PyTuple::new(py, ["longitude", "f8"])?,
                PyTuple::new(py, ["baro_altitude", "i4"])?,
                PyTuple::new(py, ["gnss_altitude", "i4"])?,
            ],
        )?;

        let dtype = numpy.getattr("dtype")?.call1((dtype_spec,))?;
        m.add("FIX_DTYPE", dtype)?;

        Ok::<(), PyErr>(())
    })?;

    Ok(())
}
