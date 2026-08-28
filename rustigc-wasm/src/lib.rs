// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! WASM bindings for rustigc.
//!
//! Values cross as plain data. Field names come from the core's serde derives, so they stay
//! `snake_case` — `distance_m`, `baro_alt` — and the method names follow them.
//!
//! Nothing crosses as a Rust map. `serde_wasm_bindgen` renders one as a `Map`, which
//! `JSON.stringify` prints as `{}` — hence `header`, one key at a time, rather than the whole
//! `headers` map.

use rustigc::{FlightDetection, FlightSelection, TrackLine};
use wasm_bindgen::prelude::*;

/// Shapes that cross as plain data, declared for the generated `.d.ts`.
///
/// They mirror the core's serde derives field for field: keep them in step.
#[wasm_bindgen]
extern "C" {
    /// A `Flight` on its way in. Naming the interface through an extern type rather than
    /// `unchecked_param_type` keeps the argument optional in the generated `.d.ts`.
    #[wasm_bindgen(typescript_type = "Flight")]
    pub type FlightArg;

    /// A `Score` on its way in, for the same reason.
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
    /** Takeoff. */
    start: number;
    /** Landing. */
    stop: number;
}

/** One header value and who entered it. */
export interface Header {
    /** The value as written, trimmed of its key. */
    text: string;
    origin: "flightrecorder" | "observer" | "pilot" | "unknown";
}

/**
 * What the winning rule of a league scored.
 *
 * Every fix is an index into the track that was scored.
 */
export interface Score {
    /** Identity of the scoring league, `"xcontest"`. */
    league: string;
    /** The rule that won, `"closed fai triangle"`. */
    description: string;
    /** Scored distance in meters, to the nearest millimeter. */
    distance_m: number;
    /** The same distance in kilometers, as the rule presents it. */
    distance_km: number;
    /** Closing leg of a circuit, in kilometers; 0 for an open task. */
    gap_km: number;
    /** Largest gap the rule would still accept, in meters; 0 for an open task. */
    threshold_m: number;
    /** What the rule charged for that gap, in points. */
    penalty: number;
    /** Final score, in league points. */
    score: number;
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
    /// About 10x faster than `track`, if you decode it yourself. The `rustigc-utils`
    /// package ships a decoder; the crate README has the layout.
    #[wasm_bindgen(getter)]
    pub fn track_bytes(&self) -> Vec<u8> {
        let track = &self.inner.track;
        // SAFETY: Fix is repr(C), the slice is valid for this borrow, and to_vec copies out
        // before anything can realloc it.
        unsafe {
            std::slice::from_raw_parts(
                track.as_ptr() as *const u8,
                std::mem::size_of_val(&track[..]),
            )
        }
        .to_vec()
    }

    /// Instant this log's fix timestamps count from as an ISO8601 String, or `undefined`
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

    /// Flight sections detected in the track, empty when none was.
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

    /// Score `window` against every rule of `league` and report the best.
    ///
    /// `window` defaults to the longest detected flight, whether left out or passed `undefined`.
    /// `undefined` when nothing could be scored; throws when `league` is not one of
    /// `league_names()`.
    #[wasm_bindgen(unchecked_return_type = "Score | undefined")]
    pub fn score(
        &self,
        league: &str,
        window: Option<FlightArg>,
    ) -> Result<JsValue, JsError> {
        known_league(league)?;

        let given: Option<rustigc::Flight> = match window {
            Some(value) => serde_wasm_bindgen::from_value(value.into())?,
            None => None,
        };

        let scored = given
            .or_else(|| self.inner.track.flights().longest().copied())
            .and_then(|w| self.inner.score(league, w.start, w.stop));

        Ok(serde_wasm_bindgen::to_value(&scored)?)
    }

    /// The log and the layers handed to it, as one GeoJSON string.
    ///
    /// `window` and `scored` may each be left out; `track` draws the flown line. Every feature
    /// declares a `role` — `track`, `marker`, `leg`, `closing`, `score` or `metadata`.
    /// `JSON.parse` it for objects.
    ///
    /// Throws when a layer is not the shape it should be, or reads a fix this log's track does
    /// not hold
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

        let collection = self.inner.export_flight(window, scored.as_ref(), line)?;

        Ok(serde_json::to_string(&collection)?)
    }

    /// Everything the log describes about itself under `league`, as one GeoJSON string.
    ///
    /// Detects the longest flight, scores it and draws both. Use `export` when the flight and
    /// score are already at hand.
    pub fn describe(&self, league: &str) -> Result<String, JsError> {
        known_league(league)?;

        Ok(serde_json::to_string(&self.inner.describe(league))?)
    }
}

/// Reject a league the registry does not hold.
fn known_league(league: &str) -> Result<(), JsError> {
    rustigc::league_names()
        .any(|name| name == league)
        .then_some(())
        .ok_or_else(|| JsError::new(&format!("unknown league {league:?}")))
}

impl Log {
    /// Fix `index`, as an error rather than the panic indexing would raise.
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
        if coords.len() % 2 != 0 {
            return Err(JsError::new(&format!(
                "{} coordinates is not a whole number of latitude, longitude pairs",
                coords.len()
            )));
        }

        let points = coords.len() / 2;
        // SAFETY: a Box<[f64]> of even length and a Box<[[f64; 2]]> of half that length describe
        // the same allocation — 8-byte align, identical size — so retyping it keeps the layout it
        // will be dropped through. A boxed slice carries no capacity of its own to reconcile.
        let table = unsafe {
            let ptr = Box::into_raw(coords) as *mut [f64; 2];
            Box::from_raw(std::slice::from_raw_parts_mut(ptr, points))
        };

        let inner = rustigc::Scorer::from_vec(table.into_vec()).ok_or_else(|| {
            JsError::new(&format!(
                "{points} points are not scorable: fewer than two, or a coordinate out of range"
            ))
        })?;

        Ok(Scorer { inner })
    }

    /// Score the table against every rule of `league` and report the best.
    ///
    /// Every fix of the result is an index into the table. `undefined` when nothing could be
    /// scored; throws when `league` is not one of `league_names()`.
    #[wasm_bindgen(unchecked_return_type = "Score | undefined")]
    pub fn solve(&self, league: &str) -> Result<JsValue, JsError> {
        known_league(league)?;

        Ok(serde_wasm_bindgen::to_value(&self.inner.solve(league))?)
    }
}

/// Every league name `score` and `describe` accept.
#[wasm_bindgen]
pub fn league_names() -> Vec<String> {
    rustigc::league_names().map(String::from).collect()
}
