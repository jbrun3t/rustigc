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
/// Keeps every record, including the ones [`Log`] drops, and checks nothing beyond each record's
/// syntax. `Display` prints it back as IGC.
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

/// Rebuilds records from a log: the A-record with `XRS` as manufacturer, the headers, the task, the
/// fixes, and a placeholder G-record. Whatever [`Log`] dropped is not restored.
impl From<Log> for RawLog<'_> {
    fn from(log: Log) -> Self {
        let mut records: Vec<Record> = Vec::new();

        let mut recorder = log.recorder;
        recorder.manufacturer = "XRS".into();

        records.push(recorder.into());

        for (key, value) in log.headers.into_iter() {
            records.push(Header { key, value }.into());
        }

        if let Some(task) = log.task {
            records.push(task.into())
        }

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

        records.push(Record::G(TextEvent {
            text: b"RUSTIGCLOGISNOTVALID",
        }));

        Self { records }
    }
}

/// Prints the records back as IGC, one per line; a [`Record::Bad`] prints as an empty line.
///
/// Bytes that are not UTF-8, which the parser accepts, come back replaced, so the output matches
/// the input only for a UTF-8 file.
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
