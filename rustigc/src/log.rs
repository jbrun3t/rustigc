// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Main Log file representation

use std::collections::HashMap;

use crate::{decode::*, Scorer, ScoringResult};
use crate::{LError, LResult, RawLog};

use log::warn;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Bare minimum to be exported when parsing an IGC
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Log {
    /// Logger identification
    pub recorder: Recorder,
    /// H-records, keyed by their 3-letter code
    pub headers: HashMap<String, HeaderData>,
    /// Position fixes (B-records), timestamps strictly increasing
    pub track: Vec<Fix>,
    /// Declared Task
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
    pub fn new(input: &[u8]) -> LResult<Self> {
        let raw = RawLog::new(input)?;
        raw.try_into()
    }

    /// Scores the fixes in `[start, stop]` against `league`'s rules
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
}
