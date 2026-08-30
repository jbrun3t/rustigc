// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Record-level IGC log.

use std::fmt;
use winnow::combinator::{alt, repeat};
use winnow::prelude::*;

use crate::decode::*;
use crate::{DecodeError, Log};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An IGC file as records, borrowing from the bytes it was parsed from.
///
/// Keeps every record, including the ones [`Log`] drops, and validates nothing beyond recognizing
/// them. `Display` renders it back as valid IGC.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(bound(deserialize = "'de: 'a")))]
pub struct RawLog<'a> {
    /// Every line of the file, in order, unreadable ones included as [`Record::Bad`].
    pub records: Vec<Record<'a>>,
}

impl<'a> RawLog<'a> {
    /// Parses `input`, an IGC file read as bytes, borrowing from it.
    ///
    /// Timestamps are carried past midnight, so a flight spanning two days keeps counting up.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Parse`] when not a single record can be read.
    pub fn new(input: &'a [u8]) -> Result<Self, DecodeError> {
        let mut offset: u32 = 0;
        let mut lastts: u32 = 0;
        let records = repeat(
            1..,
            // winnow's `Alt` tuple impl tops out at 10 elements, so 13 record kinds need splitting.
            alt((
                alt((b_record, k_record, e_record, f_record, h_record, l_record)),
                alt((
                    g_record, d_record, a_record, i_record, j_record, c_record,
                    bad_record,
                )),
            ))
            .map(|r| r.fix_timestamp(&mut offset, &mut lastts)),
        )
        .parse(input)?;

        Ok(Self { records })
    }
}

/// Renders a log back as records, with `XRS` as the manufacturer — the result is not what a
/// recorder wrote.
impl From<Log> for RawLog<'_> {
    fn from(log: Log) -> Self {
        let mut records: Vec<Record> = Vec::new();

        // Make it clear where this log comes from
        let mut recorder = log.recorder;
        recorder.manufacturer = "XRS".into();

        // The mandatory A record first
        records.push(recorder.into());

        // All the H record
        for (key, value) in log.headers.into_iter() {
            records.push(Header { key, value }.into());
        }

        // The optional C records if we have a task
        if let Some(task) = log.task {
            records.push(task.into())
        }

        // All the B records carrying the fixes
        for fix in log.track.into_iter() {
            records.push(
                RawFix {
                    fix,
                    valid: true,
                    ext: b"",
                }
                .into(),
            );
        }

        // Finally a fake G record just in case
        records.push(Record::G(TextEvent {
            text: b"RUSTIGCLOGISNOTVALID",
        }));

        Self { records }
    }
}

/// Prints back valid IGC, one record per line.
///
/// A record holding bytes that are not UTF-8 — which the parser accepts — comes back with each
/// of them replaced, so the output matches the input only for a file that was UTF-8 to begin
/// with. This should not really happen since IGC are supposed to be ASCII only ...
impl fmt::Display for RawLog<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for rec in &self.records {
            rec.fmt(f)?;
            writeln!(f)?;
        }
        Ok(())
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

        let raw = RawLog::new(content).unwrap();
        assert_eq!(raw.records.len(), 10);
    }
}
