// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! WASM bindings for rustigc.
//!
//! Everything crosses as plain JS data: there is no handle to keep alive and nothing to free.
//! Field names come from the core's serde derives, so they stay `snake_case` — `raw_distance`,
//! `baro_alt` — and the method names follow them.
//!
//! Nothing crosses as a Rust map. `serde_wasm_bindgen` renders one as a JS `Map`, which
//! `JSON.stringify` prints as `{}` — hence `header`, one key at a time, rather than the whole
//! `headers` map.

use rustigc::{FlightDetection, FlightSelection, Zoned};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// A zoned instant, handed over already split so callers never parse one.
#[derive(Serialize)]
struct DateTime {
    /// Local calendar day, `2022-08-05`.
    date: String,
    /// Local wall clock, `10:09:32`.
    time: String,
    /// The instant, `2022-08-05T10:09:32+01:00`
    iso: String,
    /// IANA name, `Europe/London`, or offset as fallback.
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
    /// Parse IGC file content.
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

    /// One header as `{text, origin}`, or `undefined` when the log has no such key.
    pub fn header(&self, key: &str) -> Result<JsValue, JsError> {
        Ok(serde_wasm_bindgen::to_value(&self.inner.headers.get(key))?)
    }

    /// The whole track, one object per fix.
    #[wasm_bindgen(getter)]
    pub fn track(&self) -> Result<JsValue, JsError> {
        Ok(serde_wasm_bindgen::to_value(&self.inner.track)?)
    }

    /// Instant this log's fix timestamps count from, or `undefined` without a usable `HFDTE`
    /// header.
    ///
    /// UTC midnight of the flight's date, in the zone the track starts in.
    pub fn datetime(&self) -> Result<JsValue, JsError> {
        let origin = self.inner.datetime();

        Ok(serde_wasm_bindgen::to_value(
            &origin.as_ref().map(DateTime::new),
        )?)
    }

    /// One fix: `{timestamp, lat, lon, baro_alt, gnss_alt}`.
    pub fn fix(&self, index: usize) -> Result<JsValue, JsError> {
        Ok(serde_wasm_bindgen::to_value(self.get(index)?)?)
    }

    /// Flight sections detected in the track, empty when none was.
    pub fn flights(&self) -> Result<JsValue, JsError> {
        Ok(serde_wasm_bindgen::to_value(&self.inner.track.flights())?)
    }

    /// The longest detected flight, or `undefined` when there is none.
    pub fn longest_flight(&self) -> Result<JsValue, JsError> {
        Ok(serde_wasm_bindgen::to_value(
            &self.inner.track.flights().longest(),
        )?)
    }

    /// Score `window` against `league`, the longest detected flight when it is left out.
    pub fn score(&self, league: &str, window: JsValue) -> Result<JsValue, JsError> {
        Self::known_league(league)?;

        let given: Option<rustigc::Flight> = serde_wasm_bindgen::from_value(window)?;

        let scored = given
            .or_else(|| self.inner.track.flights().longest().copied())
            .and_then(|w| self.inner.score(league, w.start, w.stop));

        Ok(serde_wasm_bindgen::to_value(&scored)?)
    }

        };

    }

    /// When `index` was recorded, or `undefined` when the log has no date.
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

/// Every league `score` accepts.
#[wasm_bindgen]
pub fn league_names() -> Vec<String> {
    rustigc::league_names().map(String::from).collect()
}
