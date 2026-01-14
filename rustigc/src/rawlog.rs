//! Main IGC file parser
//!
//! This module orchestrates parsing of complete IGC files by dispatching
//! each line to the appropriate record parser.

use std::fmt;
use winnow::combinator::{alt, repeat};
use winnow::prelude::*;

use crate::records::*;
use crate::{Log, Result};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(bound(deserialize = "'de: 'a")))]
pub struct RawLog<'a> {
    pub records: Vec<Record<'a>>,
}

impl<'a> RawLog<'a> {
    pub fn new(input: &'a str) -> Result<Self> {
        let mut offset: u32 = 0;
        let mut lastts: u32 = 0;
        let records = repeat(
            1..,
            alt((
                b_record, k_record, e_record, f_record, h_record, l_record, g_record,
                d_record, a_record, i_record, j_record, c_record,
            ))
            .map(|r| r.fix_timestamp(&mut offset, &mut lastts)),
        )
        .parse(input)?;

        Ok(Self { records })
    }
}

impl<'a> From<Log> for RawLog<'a> {
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
                    ext: "",
                }
                .into(),
            );
        }

        // Finally a fake G record just in case
        records.push(Record::G("RUSTIGCLOGISNOTVALID"));

        Self { records }
    }
}

impl<'a> fmt::Display for RawLog<'a> {
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
        let content = "AFLA1BX\n\
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
