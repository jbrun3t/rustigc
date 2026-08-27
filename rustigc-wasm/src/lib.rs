// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! WASM bindings for rustigc.
//!
//! Everything crosses as plain data: there is no handle to keep alive and nothing to free.
//! Field names come from the core's serde derives, so they stay `snake_case` — `raw_distance`,
//! `baro_alt` — and the method names follow them.
//!
//! Nothing crosses as a Rust map. `serde_wasm_bindgen` renders one as a `Map`, which
//! `JSON.stringify` prints as `{}` — hence `header`, one key at a time, rather than the whole
//! `headers` map.

use rustigc::{FlightDetection, FlightSelection, TrackLine, Zoned};
use serde::Serialize;
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
    /** Seconds from the instant `Log.datetime` reports. */
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

/** A zoned instant, handed over already split so callers never parse one. */
export interface DateTime {
    /** Local calendar day, `2022-08-05`. */
    date: string;
    /** Local wall clock, `10:09:32`. */
    time: string;
    /** The instant, `2022-08-05T10:09:32+01:00`. This is what `new Date()` accepts. */
    iso: string;
    /** IANA name, `Europe/London`, or an offset as fallback. */
    zone: string;
}

/**
 * What the winning rule of a league scored.
 *
 * Every fix is an index into the track that was scored.
 */
export interface Score {
    /** The rule that won, `"Closed FAI Triangle"`. */
    description: string;
    /** Scored distance, as the rule presents it. */
    distance: number;
    /** The same distance in meters, to the nearest millimeter. */
    raw_distance: number;
    /** Closing leg of a circuit, 0 for an open task. */
    gap: number;
    /** What the rule charged for that gap. */
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

/// A zoned instant, handed over already split so callers never parse one.
#[derive(Serialize)]
struct DateTime {
    /// Local calendar day, `2022-08-05`.
    date: String,
    /// Local wall clock, `10:09:32`.
    time: String,
    /// The instant, `2022-08-05T10:09:32+01:00`.
    iso: String,
    /// IANA name, `Europe/London`, or the offset as a fallback.
    zone: String,
}

impl DateTime {
    fn new(zoned: &Zoned) -> Self {
        DateTime {
            date: zoned.strftime("%Y-%m-%d").to_string(),
            time: zoned.strftime("%H:%M:%S").to_string(),
            iso: zoned.strftime("%Y-%m-%dT%H:%M:%S%:z").to_string(),
            zone: zoned.strftime("%:Q").to_string(),
        }
    }
}

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

    /// Instant this log's fix timestamps count from, or `undefined` without a usable `HFDTE`
    /// header.
    ///
    /// UTC midnight of the flight's date, in the zone the track starts in. Add a fix's
    /// `timestamp` seconds to it to get when that fix was recorded.
    #[wasm_bindgen(unchecked_return_type = "DateTime | undefined")]
    pub fn datetime(&self) -> Result<JsValue, JsError> {
        let origin = self.inner.datetime();

        Ok(serde_wasm_bindgen::to_value(
            &origin.as_ref().map(DateTime::new),
        )?)
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
        Self::known_league(league)?;

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

        let collection = self.inner.export_flight(window, scored.as_ref(), line);

        Ok(serde_json::to_string(&collection)?)
    }

    /// Everything the log describes about itself under `league`, as one GeoJSON string.
    ///
    /// Detects the longest flight, scores it and draws both. Use `export` when the flight and
    /// score are already at hand.
    pub fn describe(&self, league: &str) -> Result<String, JsError> {
        Self::known_league(league)?;

        Ok(serde_json::to_string(&self.inner.describe(league))?)
    }

    /// When fix `index` was recorded, or `undefined` when the log has no date.
    #[wasm_bindgen(unchecked_return_type = "DateTime | undefined")]
    pub fn fix_datetime(&self, index: usize) -> Result<JsValue, JsError> {
        let fix = self.get(index)?;
        let stamp = self.inner.datetime().map(|origin| fix.datetime(&origin));

        Ok(serde_wasm_bindgen::to_value(
            &stamp.as_ref().map(DateTime::new),
        )?)
    }
}

impl Log {
    /// Reject a league the registry does not hold.
    fn known_league(league: &str) -> Result<(), JsError> {
        rustigc::league_names()
            .any(|name| name == league)
            .then_some(())
            .ok_or_else(|| JsError::new(&format!("unknown league {league:?}")))
    }

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

/// Every league name `score` and `describe` accept.
#[wasm_bindgen]
pub fn league_names() -> Vec<String> {
    rustigc::league_names().map(String::from).collect()
}
