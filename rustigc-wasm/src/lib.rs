// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! WASM bindings for rustigc.
//!
//! Values cross as plain data with the core's `snake_case` field names, and the method names
//! follow them. No map crosses: `serde_wasm_bindgen` makes one a `Map`, which `JSON.stringify`
//! prints as `{}`, so headers are read one key at a time with `header`.

use rustigc::{FlightDetection, FlightSelection, TrackLine};
use wasm_bindgen::prelude::*;

/// Types for the generated `.d.ts`. They mirror the core's serde derives field for field.
#[wasm_bindgen]
extern "C" {
    /// A `Flight` argument; an extern type keeps it optional in the `.d.ts`.
    #[wasm_bindgen(typescript_type = "Flight")]
    pub type FlightArg;

    /// A `Score` argument, likewise.
    #[wasm_bindgen(typescript_type = "Score")]
    pub type ScoreArg;
}

#[wasm_bindgen(typescript_custom_section)]
const TYPES: &'static str = r#"
/** One position fix. */
export interface Fix {
    /** Milliseconds from the instant `Log.datetime` reports. */
    timestamp: number;
    /** Latitude in decimal degrees, north positive. */
    lat: number;
    /** Longitude in decimal degrees, east positive. */
    lon: number;
    /** Pressure altitude in meters. */
    baro_alt: number;
    /** GNSS altitude in meters. */
    gnss_alt: number;
}

/** One flight section, as fix indices into the track it was detected in. */
export interface Flight {
    /** First fix of the flight. */
    start: number;
    /** Last fix of the flight. */
    stop: number;
}

/** One header value and who entered it. */
export interface Header {
    /** The value as written, trimmed of its key. */
    text: string;
    origin: "flightrecorder" | "observer" | "pilot" | "unknown";
}

/**
 * The winning rule's result. Every fix is an index into the track that was scored.
 */
export interface Score {
    /** Name of the scoring ruleset, `"xcontest"`. */
    ruleset: string;
    /** Name of the winning variant, `"closed fai triangle"`. */
    description: string;
    /** Scored distance in meters, net of the penalty, to the nearest millimeter. */
    distance_m: number;
    /** Closing leg of a circuit, in meters; 0 for an open task. */
    gap_m: number;
    /** Largest gap the winning variant closes, in meters; 0 for an open task. */
    threshold_m: number;
    /** Scored distance in kilometers. */
    distance_km: number;
    /** Final score, in the ruleset's points. */
    score: number;
    /** What the rule charged for that gap, in kilometers off the distance. */
    penalty: number;
    /** Multiplier the rule scored at. */
    multiplier: number;
    /** Start of the scored window. */
    takeoff: number;
    /** First fix of the task. */
    entry: number;
    /** Turnpoints of the task, in order. */
    turnpoints: number[];
    /** Last fix of the task. */
    exit: number;
    /** End of the scored window. */
    landing: number;
    /** Whether the task closes on itself. */
    circuit: boolean;
}
"#;

/// A parsed IGC log.
#[wasm_bindgen]
pub struct Log {
    inner: rustigc::Log,
}

#[wasm_bindgen]
impl Log {
    /// Parse IGC file content. Throws when the bytes are not usable IGC.
    #[wasm_bindgen(constructor)]
    pub fn new(content: &[u8]) -> Result<Log, JsError> {
        let inner = rustigc::Log::new(content)
            .map_err(|e| JsError::new(&format!("Failed to parse IGC file: {e}")))?;

        Ok(Log { inner })
    }

    /// Number of fixes in the track.
    #[wasm_bindgen(getter)]
    pub fn fix_count(&self) -> usize {
        self.inner.track.len()
    }

    /// Every 3-letter code this log carries a header for.
    #[wasm_bindgen(getter)]
    pub fn header_keys(&self) -> Vec<String> {
        self.inner.headers.keys().cloned().collect()
    }

    /// One header, or `undefined` when the log has no such key.
    ///
    /// `key` is a 3-letter code: `"PLT"` for the pilot, `"GTY"` for the glider, `"DTE"` for the
    /// date, ... `header_keys` lists the ones this log carries.
    #[wasm_bindgen(unchecked_return_type = "Header | undefined")]
    pub fn header(&self, key: &str) -> Result<JsValue, JsError> {
        Ok(serde_wasm_bindgen::to_value(&self.inner.headers.get(key))?)
    }

    /// The whole track, one object per fix.
    #[wasm_bindgen(getter, unchecked_return_type = "Fix[]")]
    pub fn track(&self) -> Result<JsValue, JsError> {
        Ok(serde_wasm_bindgen::to_value(&self.inner.track)?)
    }

    /// The same track as raw `#[repr(C)] Fix` bytes, 32 per fix, little-endian.
    ///
    /// About 10x faster than `track` once decoded. `rustigc-wasm-utils` ships a decoder; the
    /// README has the layout.
    #[wasm_bindgen(getter)]
    pub fn track_bytes(&self) -> Vec<u8> {
        let track = &self.inner.track;
        // SAFETY: `Fix` is `repr(C)`, and `to_vec` copies the slice while the borrow holds.
        unsafe {
            std::slice::from_raw_parts(
                track.as_ptr() as *const u8,
                std::mem::size_of_val(&track[..]),
            )
        }
        .to_vec()
    }

    /// Instant this log's fix timestamps count from, as an ISO 8601 string, or `undefined`
    /// without a usable `HFDTE` header.
    pub fn datetime(&self) -> Option<String> {
        self.inner.datetime()
    }

    /// Offset from UTC to local time in hours, as the recorder declared it in `TZN`, or
    /// `undefined` when it declared none.
    pub fn tzn(&self) -> Option<f64> {
        self.inner.tzn()
    }

    /// One fix. Throws when `index` is past the end of the track.
    #[wasm_bindgen(unchecked_return_type = "Fix")]
    pub fn fix(&self, index: usize) -> Result<JsValue, JsError> {
        Ok(serde_wasm_bindgen::to_value(self.get(index)?)?)
    }

    /// Flight sections detected in the track, empty when none is found.
    #[wasm_bindgen(unchecked_return_type = "Flight[]")]
    pub fn flights(&self) -> Result<JsValue, JsError> {
        Ok(serde_wasm_bindgen::to_value(&self.inner.track.flights())?)
    }

    /// The longest detected flight by fix span, or `undefined` when there is none.
    #[wasm_bindgen(unchecked_return_type = "Flight | undefined")]
    pub fn longest_flight(&self) -> Result<JsValue, JsError> {
        Ok(serde_wasm_bindgen::to_value(
            &self.inner.track.flights().longest(),
        )?)
    }

    /// Score `window` against every rule of `ruleset` and report the best.
    ///
    /// `window` defaults to the longest detected flight. `undefined` when no rule scores; throws
    /// when the window is not one this track holds, or when no flight is detected to default to.
    #[wasm_bindgen(unchecked_return_type = "Score | undefined")]
    pub fn score(
        &self,
        ruleset: &Ruleset,
        window: Option<FlightArg>,
    ) -> Result<JsValue, JsError> {
        let given: Option<rustigc::Flight> = match window {
            Some(value) => serde_wasm_bindgen::from_value(value.into())?,
            None => None,
        };

        let window = given
            .or_else(|| self.inner.track.flights().longest().copied())
            .ok_or_else(|| JsError::new("no flight detected, pass an explicit window"))?;

        let scored = self
            .inner
            .score(&ruleset.inner, window.start, window.stop)?;

        Ok(serde_wasm_bindgen::to_value(&scored)?)
    }

    /// GeoJSON of the log, drawing `window` and `scored` when given and the track line unless
    /// `track` is `false`. Every feature has a `role`: `track`, `marker`, `leg`, `closing`,
    /// `score` or `metadata`.
    ///
    /// Throws when a layer is not the shape it should be, or holds a fix index past the end of
    /// the track.
    pub fn export(
        &self,
        window: Option<FlightArg>,
        scored: Option<ScoreArg>,
        track: Option<bool>,
    ) -> Result<String, JsError> {
        let window: Option<rustigc::Flight> = match window {
            Some(value) => serde_wasm_bindgen::from_value(value.into())?,
            None => None,
        };
        let scored: Option<rustigc::ScoringResult> = match scored {
            Some(value) => serde_wasm_bindgen::from_value(value.into())?,
            None => None,
        };

        let line = match track.unwrap_or(true) {
            true => TrackLine::Draw,
            false => TrackLine::Skip,
        };

        let collection = self.inner.export(window, scored.as_ref(), line)?;

        Ok(serde_json::to_string(&collection)?)
    }

    /// GeoJSON of the track, its longest detected flight and that flight's score under
    /// `ruleset`. Use `export` when the flight and score are already at hand.
    pub fn describe(&self, ruleset: &Ruleset) -> Result<String, JsError> {
        Ok(serde_json::to_string(
            &self.inner.describe(&ruleset.inner)?,
        )?)
    }
}

impl Log {
    /// Fix `index`, or an error past the end of the track.
    fn get(&self, index: usize) -> Result<&rustigc::Fix, JsError> {
        self.inner.track.get(index).ok_or_else(|| {
            JsError::new(&format!(
                "fix {index} is out of range, the track holds {}",
                self.inner.track.len()
            ))
        })
    }
}

/// A scoring window over a table of coordinates, needing no `Log`.
#[wasm_bindgen]
pub struct Scorer {
    inner: rustigc::Scorer,
}

#[wasm_bindgen]
impl Scorer {
    /// Prepare `coords`, interleaved latitude and longitude in decimal degrees, in flight order.
    ///
    /// Throws unless it holds at least two whole pairs of coordinates that are all in range.
    #[wasm_bindgen(constructor)]
    pub fn new(coords: Box<[f64]>) -> Result<Scorer, JsError> {
        if !coords.len().is_multiple_of(2) {
            return Err(JsError::new(&format!(
                "{} coordinates is not a whole number of latitude, longitude pairs",
                coords.len()
            )));
        }

        let points = coords.len() / 2;
        // SAFETY: a `Box<[f64]>` of even length and a `Box<[[f64; 2]]>` of half that length have
        // the same size and alignment, so the allocation is dropped with the layout it has.
        let table = unsafe {
            let ptr = Box::into_raw(coords) as *mut [f64; 2];
            Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, points))
        };

        let inner = rustigc::Scorer::from_vec(table.into_vec())?;

        Ok(Scorer { inner })
    }

    /// Score the table against every rule of `ruleset` and report the best.
    ///
    /// Every fix of the result is an index into the table. `undefined` when no rule scores.
    #[wasm_bindgen(unchecked_return_type = "Score | undefined")]
    pub fn solve(&self, ruleset: &Ruleset) -> Result<JsValue, JsError> {
        Ok(serde_wasm_bindgen::to_value(
            &self.inner.solve(&ruleset.inner),
        )?)
    }
}

/// A set of rules to score against: a preset, or a caller's own in the presets' JSON format.
#[wasm_bindgen]
pub struct Ruleset {
    inner: rustigc::Ruleset,
}

#[wasm_bindgen]
impl Ruleset {
    /// The preset `id`, one of `preset_names()`; throws when there is none.
    pub fn preset(id: &str) -> Result<Ruleset, JsError> {
        Ok(Ruleset {
            inner: rustigc::Ruleset::preset(id)?,
        })
    }

    /// A ruleset from JSON content in the presets' format; throws, with the line and column,
    /// when it is not one.
    pub fn from_json(text: &str) -> Result<Ruleset, JsError> {
        Ok(Ruleset {
            inner: rustigc::Ruleset::from_json(text)?,
        })
    }

    /// The name reported in every result this ruleset scores.
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }
}

/// Every preset id `Ruleset.preset` accepts.
#[wasm_bindgen]
pub fn preset_names() -> Vec<String> {
    rustigc::preset_names().map(String::from).collect()
}
