// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Decoded IGC log.

use std::collections::HashMap;

use crate::{decode::*, ScoreError, Scorer, ScoringResult};
use crate::{DecodeError, RawLog};

use log::warn;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Share of invalid records at or above which a file is rejected, in tenths.
const BAD_RECORD_LIMIT: usize = 8;

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
    type Error = DecodeError;

    fn try_from(raw: RawLog) -> Result<Self, Self::Error> {
        let count = raw.records.len();
        if count == 0 {
            return Err(DecodeError::Empty);
        }

        let mut headers = HashMap::new();
        let mut track = Vec::new();
        let mut task: Option<Task> = None;
        let mut recorder: Option<Recorder> = None;
        let mut _bextensions: Option<Vec<RecordExtension>> = None;
        let mut bad_count: usize = 0;
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

        // Reject a file that is 80% bad or worse
        // TODO: Try to reject earlier, while parsing
        if (bad_count * 10) / count >= BAD_RECORD_LIMIT {
            return Err(DecodeError::TooManyBadRecords);
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
    /// [`DecodeError::Parse`] when no record can be read at all,
    /// [`DecodeError::TooManyBadRecords`] when at least 80% of them are invalid.
    pub fn new(input: &[u8]) -> Result<Self, DecodeError> {
        let raw = RawLog::new(input)?;
        raw.try_into()
    }

    /// Scores the fixes in `[start, stop]` against every rule of `league`, reporting the best.
    ///
    /// The window is a pair of indices into [`Log::track`]; flight detection is the usual source
    /// of one.
    ///
    /// `None` when no rule could score the window — a real answer, not a failure.
    ///
    /// # Errors
    ///
    /// [`ScoreError::UnknownLeague`] when `league` is not one of [`league_names`],
    /// [`ScoreError::Track`] when the window is not one this log's track holds.
    ///
    /// [`league_names`]: crate::league_names
    pub fn score(
        &self,
        league: &str,
        start: usize,
        stop: usize,
    ) -> Result<Option<ScoringResult>, ScoreError> {
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
        assert_eq!(log.track[0].timestamp, 39_695_000);
    }

    /// A `RawLog` can be built or deserialized empty, and the bad-record ratio divides by its
    /// record count.
    #[test]
    fn log_from_recordless_rawlog() {
        let raw = RawLog {
            records: Vec::new(),
        };

        assert_eq!(Log::try_from(raw).unwrap_err(), DecodeError::Empty);
    }
}
