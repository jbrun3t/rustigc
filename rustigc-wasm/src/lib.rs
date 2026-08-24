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

use wasm_bindgen::prelude::*;

fn js_err(e: impl std::fmt::Display) -> JsError {
    JsError::new(&e.to_string())
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
            .map_err(|e| js_err(format!("Failed to parse IGC file: {e}")))?;

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
        match self.inner.headers.get(key) {
            Some(data) => serde_wasm_bindgen::to_value(data).map_err(js_err),
            None => Ok(JsValue::UNDEFINED),
        }
    }

    /// The whole track, one object per fix.
    #[wasm_bindgen(getter)]
    pub fn track(&self) -> Result<JsValue, JsError> {
        serde_wasm_bindgen::to_value(&self.inner.track).map_err(js_err)
    }

    /// Instant this log's fix timestamps count from, or `undefined` without a usable `HFDTE`
    /// header.
    ///
    /// UTC midnight of the flight's date, in the zone the track starts in, as RFC 9557 —
    /// `2022-08-05T01:00:00+01:00[Europe/London]`. That is what a `Zoned` prints, offset and zone
    /// name together, so the other side can rebuild the zone itself rather than be handed pieces.
    pub fn datetime(&self) -> Option<String> {
        self.inner.datetime().map(|origin| origin.to_string())
    }

    /// One fix: `{timestamp, lat, lon, baro_alt, gnss_alt}`.
    pub fn fix(&self, index: usize) -> Result<JsValue, JsError> {
        serde_wasm_bindgen::to_value(self.get(index)?).map_err(js_err)
    }

    /// When `index` was recorded, as RFC 9557, or `undefined` when the log has no date.
    ///
    /// Here rather than in JS so the zone maths stays in `jiff`: callers only ever slice the
    /// result.
    pub fn fix_datetime(&self, index: usize) -> Result<Option<String>, JsError> {
        let fix = self.get(index)?;

        Ok(self
            .inner
            .datetime()
            .map(|origin| fix.datetime(&origin).to_string()))
    }
}

impl Log {
    /// Fix `index`, as an error rather than the panic indexing would raise.
    fn get(&self, index: usize) -> Result<&rustigc::Fix, JsError> {
        self.inner.track.get(index).ok_or_else(|| {
            js_err(format!(
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
