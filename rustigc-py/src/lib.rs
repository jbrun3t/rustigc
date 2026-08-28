// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

#![allow(clippy::useless_conversion)]

use ::rustigc;
use numpy::{PyReadonlyArray2, PyUntypedArrayMethods};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList, PyTuple};
use rustigc::FlightDetection;

/// A parsed IGC log, as Rust holds it.
///
/// The raw binding behind `rustigcpy.Log`, which is the interface to use.
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
        let inner = py
            .allow_threads(|| rustigc::Log::new(content))
            .map_err(|e| {
                PyValueError::new_err(format!("Failed to parse IGC file: {e}"))
            })?;

        Ok(PyLog { inner })
    }

    /// The track as raw bytes, laid out for `FIX_DTYPE`: 32 per fix, padding included.
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

    /// This log over `data` as a track, laid out for `FIX_DTYPE`: same recorder, headers, task.
    ///
    /// A new log: this one is untouched, so anything detected or scored in it still refers to the
    /// track it was built from. `data` need not hold as many fixes.
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
        let track: Vec<rustigc::Fix> = py.allow_threads(|| {
            let mut track = Vec::<rustigc::Fix>::with_capacity(count);
            // SAFETY: reinterpreting `data` as `Fix`es is safe because:
            // - Fix is repr(C), so its layout is still guaranteed
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

        // Make sure the timestamps are strictly increasing
        if let Some(w) = track.windows(2).find(|w| w[1].timestamp <= w[0].timestamp) {
            return Err(PyValueError::new_err(format!(
                "timestamps must be strictly increasing, got {} then {}",
                w[0].timestamp, w[1].timestamp
            )));
        }

        // Field by field rather than `clone()`: cloning the log would copy the old track only to
        // drop it, and a track is the one big allocation here.
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
    /// UTC midnight of the flight's date, ISO 8601 — `2022-08-05T00:00:00.000Z`. Naming the zone
    /// is the Python side's job; it has its own dataset for that.
    #[pyo3(text_signature = "($self)")]
    fn datetime(&self) -> Option<String> {
        self.inner.datetime()
    }

    /// Offset from UTC to local time in hours, as the `TZN` header declares it, or `None`.
    #[pyo3(text_signature = "($self)")]
    fn tzn(&self) -> Option<f64> {
        self.inner.tzn()
    }

    /// Detect the flight sections, as one JSON array, empty when none was detected.
    #[pyo3(text_signature = "($self)")]
    fn flights(&self, py: Python<'_>) -> PyResult<String> {
        let flights = py.allow_threads(|| self.inner.track.flights());

        serde_json::to_string(&flights).map_err(|e| {
            PyValueError::new_err(format!("Failed to serialize flights: {e}"))
        })
    }

    /// Score the fixes in `[start, stop]` against every rule of `league`, reporting the best.
    ///
    /// `None` when the league is unknown, the window unusable, or nothing could be scored.
    #[pyo3(text_signature = "($self, league, start, stop)")]
    fn score(
        &self,
        py: Python<'_>,
        league: &str,
        start: usize,
        stop: usize,
    ) -> PyResult<Option<String>> {
        // FIXME: `None` covers every way this can fail: unknown league, bad window, nothing scored.
        // Something to fixme in the core
        let result = py.allow_threads(|| self.inner.score(league, start, stop));

        result
            .map(|scored| serde_json::to_string(&scored))
            .transpose()
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize score: {e}")))
    }

    /// Everything the log describes about itself under `league`, as one GeoJSON string.
    ///
    /// Detects the longest flight and scores it.
    #[pyo3(text_signature = "($self, league)")]
    fn describe(&self, py: Python<'_>, league: &str) -> PyResult<String> {
        py.allow_threads(|| serde_json::to_string(&self.inner.describe(league)))
            .map_err(|e| {
                PyValueError::new_err(format!("Failed to serialize geojson: {e}"))
            })
    }

    /// The log, its time reference, `flight` and `score`, as one GeoJSON string.
    ///
    /// Each layer is the JSON its Python wrapper carries, read back into the struct that draws it.
    /// `track` draws the flown line. Fix indices are taken on trust: nothing checks that a layer
    /// was detected or scored in the track this log holds.
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

        py.allow_threads(|| {
            let collection = self
                .inner
                .export_flight(flight, score.as_ref(), line)
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
/// The raw binding behind `rustigcpy.Scorer`, which is the interface to use.
#[pyclass(name = "RustScorer")]
struct PyScorer {
    inner: rustigc::Scorer,
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
        // An (N, 3) of [lat, lon, alt] holds an even number of floats, so the column count is
        // what rules it out, not the size.
        let &[rows, 2] = points.shape() else {
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

        // `as_chunks` is a view, so the only copy is the one `Scorer` then owns.
        let (table, _) = flat.as_chunks::<2>();

        let inner = rustigc::Scorer::from_vec(table.to_vec()).ok_or_else(|| {
            PyValueError::new_err(format!(
                "{rows} points are not scorable: fewer than two, or a coordinate out of range"
            ))
        })?;

        Ok(Self { inner })
    }

    /// Score the table against every rule of `league` and report the best.
    ///
    /// `None` when the league is unknown, or nothing could be scored. Every fix of the result is
    /// an index into the table.
    #[pyo3(text_signature = "($self, league)")]
    fn score(&mut self, py: Python<'_>, league: &str) -> PyResult<Option<String>> {
        // `&mut`, not `&`: a `Scorer` is `Send` but not `Sync` — its caches are `RefCell` — so a
        // shared reference cannot cross `allow_threads` and an exclusive one can.
        let scorer: &mut rustigc::Scorer = &mut self.inner;

        py.allow_threads(move || scorer.solve(league))
            .map(|scored| serde_json::to_string(&scored))
            .transpose()
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize score: {e}")))
    }
}

/// Every league name `score` and `describe` accept.
#[pyfunction]
#[pyo3(text_signature = "()")]
fn league_names() -> Vec<&'static str> {
    rustigc::league_names().collect()
}

/// Raw bindings to the rustigc library.
///
/// Not the interface to use: `rustigcpy` wraps this with numpy tracks and Python objects. It holds
/// `RustLog`, `RustScorer`, `league_names` and `FIX_DTYPE`. Flights and scores cross as JSON.
#[pymodule]
#[pyo3(name = "_bindings")]
fn rustigc_py_bindings(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<PyLog>()?;
    m.add_class::<PyScorer>()?;
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
