// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! A-record (flight recorder identification) parser
//!
//! A-record format: AMMMSSSSS...
//! - A: Record type
//! - MMM: 3-character manufacturer code (e.g., FLA, GCS, CAM)
//! - SSSSS...: Serial number or logger ID (variable length)
//!
//! Example: AFLA1BX = FLA manufacturer, ID 1BX

use std::fmt;

use winnow::combinator::delimited;
use winnow::error::Result as PResult;
use winnow::prelude::*;

use super::Record;
use super::utils::{n_alphanum, robust_ending_eof, till_robust_ending};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The flight recorder that wrote the file (A-record).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Recorder {
    /// Three-letter manufacturer code, `"FLA"`, `"XCT"`, ... `"BAD"` when the file had no
    /// A-record at all.
    pub manufacturer: String,
    /// Serial number or logger id.
    pub uid: String,
    /// Whatever the manufacturer appended past the id.
    pub data: Option<String>,
}

impl fmt::Display for Recorder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self
            .data
            .as_ref()
            .map_or(if f.alternate() { "none" } else { "" }, |s| s.as_str());

        if f.alternate() {
            write!(f, "{} - {} - {}", self.manufacturer, self.uid, data)
        } else {
            write!(f, "{}{}{}", self.manufacturer, self.uid, data)
        }
    }
}

impl From<Recorder> for Record<'_> {
    fn from(v: Recorder) -> Self {
        Record::A(v)
    }
}

pub fn a_record<'a>(input: &mut &[u8]) -> PResult<Record<'a>> {
    delimited(b'A', (n_alphanum(3), till_robust_ending), robust_ending_eof)
        .map(|(m, u): (&[u8], &[u8])| {
            Recorder {
                manufacturer: String::from_utf8_lossy(m).into_owned(),
                uid: String::from_utf8_lossy(u).into_owned(),
                data: None,
            }
            .into()
        })
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recorder_minimal() {
        if let Record::A(rec) = a_record.parse(b"AFLA1BX\n").unwrap() {
            assert_eq!(rec.manufacturer, "FLA");
            assert_eq!(rec.uid, "1BX");
        } else {
            panic!()
        }
    }

    #[test]
    fn test_recorder_longer_uid() {
        if let Record::A(rec) = a_record.parse(b"AGCS0123456789ABCD\n").unwrap() {
            assert_eq!(rec.manufacturer, "GCS");
            assert_eq!(rec.uid, "0123456789ABCD");
        } else {
            panic!()
        }
    }

    #[test]
    fn test_recorder_invalid_manufacturer() {
        assert!(a_record.parse(b"AF A1BX\n").is_err());
    }

    #[test]
    fn test_identity() {
        let line = b"AFLA1BX\n";
        if let Record::A(rec) = a_record.parse(line).unwrap() {
            let formatted = format!("{}\n", rec);
            assert_eq!(formatted.as_bytes(), &line[1..]);
        } else {
            panic!()
        };
    }

    #[test]
    fn test_recorder_crazy_carriage() {
        if let Record::A(rec) = a_record.parse(b"AXSLGiveMeMyCarriageBack\r\r\n").unwrap()
        {
            assert_eq!(rec.manufacturer, "XSL");
        } else {
            panic!()
        }
    }
}
