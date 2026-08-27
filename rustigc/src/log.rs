// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Decoded IGC log.

use std::collections::HashMap;

use crate::{decode::*, Scorer, ScoringResult};
use crate::{LError, LResult, RawLog};

use log::warn;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An IGC file, decoded.
///
/// Everything the file states about the flight. Comments, signatures and events are dropped;
/// [`RawLog`] keeps them. Build one with [`Log::new`], or from a [`RawLog`] when the record-level
/// view is needed first.
///
/// Invalid fixes and fixes whose timestamp does not advance are dropped, so `track` is always
/// strictly increasing in time. A file that is mostly unparsable is rejected outright.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Log {
    /// Flight recorder that produced the file (A-record), synthesized when it is missing.
    pub recorder: Recorder,
    /// Flight metadata (H-records), keyed by 3-letter code: `"PLT"`, `"GTY"`, `"DTE"`, ...
    pub headers: HashMap<String, HeaderData>,
    /// Position fixes (B-records), in order, timestamps strictly increasing.
    pub track: Vec<Fix>,
    /// Task the pilot declared before the flight (C-records), if any.
    pub task: Option<Task>,
}

impl TryFrom<RawLog<'_>> for Log {
    type Error = LError;

    fn try_from(raw: RawLog) -> Result<Self, Self::Error> {
        let mut headers = HashMap::new();
        let mut track = Vec::new();
        let mut task: Option<Task> = None;
        let mut recorder: Option<Recorder> = None;
        let mut _bextensions: Option<Vec<RecordExtension>> = None;
        let mut bad_count: usize = 0;
        let count = raw.records.len();
        // -1 so the first fix passes whatever its timestamp
        let mut last_ts: i64 = -1;
        let mut dropped: usize = 0;

        for rec in raw.records.into_iter() {
            match rec {
                Record::A(inner) => {
                    recorder = Some(inner);
                }
                Record::B(inner) => {
                    let ts = inner.fix.timestamp as i64;
                    if !inner.valid || ts <= last_ts {
                        dropped += 1;
                        continue;
                    }
                    last_ts = ts;

                    // TODO: Handle LOD/LAD, Possibly TDS
                    track.push(inner.fix);
                }
                Record::C(inner) => {
                    task = Some(inner);
                }
                Record::H(inner) => {
                    headers.insert(inner.key, inner.value);
                }
                Record::I(inner) => {
                    _bextensions = Some(inner.vext);
                }
                Record::Bad(_) => bad_count += 1,
                _ => {}
            }
        }

        // Reject a file that is more than 80% bad
        // TODO: Try to reject earlier, while parsing
        if ((bad_count * 10) / count) >= 8 {
            return Err(LError::Doh(format!(
                "Invalid content: {bad_count} bad records"
            )));
        }

        if dropped > 0 {
            warn!("Dropped {dropped} invalid or out-of-order fix(es)");
        }

        // How can one miss this in the IGC spec ?
        // it is literally the first thing !
        let recorder = recorder.unwrap_or_else(|| {
            warn!("Missing Record \"A\". Please fix your flight recorder");
            Recorder {
                manufacturer: "BAD".into(),
                uid: "123456789ABC".into(),
                data: None,
            }
        });

        Ok(Self {
            recorder,
            headers,
            track,
            task,
        })
    }
}

impl Log {
    /// Parses `input`, an IGC file read as bytes.
    ///
    /// # Errors
    ///
    /// [`LError::Parse`] when no record can be read at all, [`LError::Doh`] when more than 80% of
    /// the records are invalid.
    pub fn new(input: &[u8]) -> LResult<Self> {
        let raw = RawLog::new(input)?;
        raw.try_into()
    }

    /// Creates a log from an existing track of [`Fix`]es.
    ///
    /// Fixes with non-increasing timestamps are dropped.
    pub fn from_track(track: Vec<Fix>) -> Self {
        let mut filtered = Vec::with_capacity(track.len());
        let mut last_ts: i64 = -1;
        for fix in track {
            let ts = fix.timestamp as i64;
            if ts <= last_ts {
                continue;
            }
            last_ts = ts;
            filtered.push(fix);
        }
        Self {
            recorder: Recorder {
                manufacturer: String::new(),
                uid: String::new(),
                data: None,
            },
            headers: HashMap::new(),
            track: filtered,
            task: None,
        }
    }

    /// Scores the fixes in `[start, stop]` against every rule of `league`, reporting the best.
    ///
    /// The window is a pair of indices into [`Log::track`]; flight detection is the usual source
    /// of one.
    ///
    /// `None` when `league` is not one of [`league_names`], when the window is unusable, or when
    /// no rule could score it.
    ///
    /// [`league_names`]: crate::league_names
    pub fn score(
        &self,
        league: &str,
        start: usize,
        stop: usize,
    ) -> Option<ScoringResult> {
        Scorer::new(&self.track, start, stop)?.solve(league)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_igc_file() {
        let content = b"AFLA1BX\n\
                       HFDTE150120\n\
                       HFPLTPILOT:Tripoux Robert\n\
                       HFGTYGLIDERTYPE:Piegon 12\n\
                       I023638FXA3940SIU\n\
                       B1101355206343N00006198WA005870055801005\n\
                       LXXX This is a comment\n\
                       B1101365206345N00006200WA005890056004208\n\
                       B1101375206347N00006202WA005910056200212\n\
                       GNOWAYINHELLTHISISVALID\n";

        let log = Log::new(content).unwrap();

        assert_eq!(log.recorder.manufacturer, "FLA");
        assert_eq!(log.headers["DTE"].text, "150120");
        assert_eq!(log.track.len(), 3);
        assert_eq!(log.track[0].timestamp, 39695);
    }

    #[test]
    fn test_log_from_track() {
        let fixes = vec![
            Fix {
                timestamp: 100,
                lat: 45.0,
                lon: 6.0,
                baro_alt: 1000,
                gnss_alt: 1050,
            },
            Fix {
                timestamp: 100, // duplicate timestamp, should be dropped
                lat: 45.1,
                lon: 6.1,
                baro_alt: 1010,
                gnss_alt: 1060,
            },
            Fix {
                timestamp: 105,
                lat: 45.2,
                lon: 6.2,
                baro_alt: 1020,
                gnss_alt: 1070,
            },
        ];

        let log = Log::from_track(fixes);
        assert_eq!(log.track.len(), 2);
        assert_eq!(log.track[0].timestamp, 100);
        assert_eq!(log.track[1].timestamp, 105);
    }
}
