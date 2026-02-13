//! A-record (flight recorder identification) parser
//!
//! A-record format: AMMMSSSSS...
//! - A: Record type
//! - MMM: 3-character manufacturer code (e.g., FLA, GCS, CAM)
//! - SSSSS...: Serial number or logger ID (variable length)
//!
//! Example: AFLA1BX = FLA manufacturer, ID 1BX

use std::fmt;

use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::delimited;
use winnow::error::Result as PResult;
use winnow::prelude::*;

use super::utils::n_alphanum;
use super::Record;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Flight Recorder Indentification
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Recorder {
    pub manufacturer: String,
    pub uid: String,
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

impl<'a> From<Recorder> for Record<'a> {
    fn from(v: Recorder) -> Self {
        Record::A(v)
    }
}

pub fn a_record<'a>(input: &mut &[u8]) -> PResult<Record<'a>> {
    delimited(b'A', (n_alphanum(3), till_line_ending), line_ending)
        .map(|(m, u): (&[u8], &[u8])| {
            Recorder {
                manufacturer: std::str::from_utf8(m).unwrap().to_string(),
                uid: std::str::from_utf8(u).unwrap().to_string(),
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
            assert!(false)
        }
    }

    #[test]
    fn test_recorder_longer_uid() {
        if let Record::A(rec) = a_record.parse(b"AGCS0123456789ABCD\n").unwrap() {
            assert_eq!(rec.manufacturer, "GCS");
            assert_eq!(rec.uid, "0123456789ABCD");
        } else {
            assert!(false)
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
            assert!(false)
        };
    }
}
