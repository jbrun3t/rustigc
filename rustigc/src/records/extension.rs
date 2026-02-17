//! I/J-record extension definition
//!
//! I/J records define additional data fields in B/K-records beyond the standard format.
//! Format: Inn[start_byte][end_byte][3-letter code]...
//! - I: Record type
//! - nn: Number of extensions (2 digits)
//! - For each extension: start byte (2 digits), end byte (2 digits), 3-letter code
//! - Note: the position provided are 1 based, and include the end character. It is
//!         stored with index 0-based, excluding the end-character
//!
//! Example: I033638FXA3940SIU4143ENL
//! - 03 extensions
//! - Bytes 36-38: FXA (fix accuracy)
//! - Bytes 39-40: SIU (satellites in use)
//! - Bytes 41-43: ENL (engine noise level)

use std::fmt;

use winnow::ascii::line_ending;
use winnow::combinator::{delimited, repeat};
use winnow::error::Result as PResult;
use winnow::prelude::*;

use super::utils::{n_alphanum, n_digits};
use super::Record;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(bound(deserialize = "'de: 'a")))]
pub struct RecordExtension<'a> {
    /// Start byte position
    pub start: usize,
    /// End byte position
    pub finish: usize,
    /// Three-letter code for this extension (e.g., FXA, SIU, ENL)
    pub tlc: &'a [u8],
    /// Correction offset in recorded data
    pub offset: usize,
}

impl fmt::Display for RecordExtension<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tlc = std::str::from_utf8(self.tlc).unwrap();
        if f.alternate() {
            write!(f, "{}: {}->{}", tlc, self.start, self.finish)
        } else {
            write!(
                f,
                "{:02}{:02}{}",
                self.start + self.offset + 1,
                self.finish + self.offset,
                tlc
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(bound(deserialize = "'de: 'a")))]
pub struct Extensions<'a> {
    pub vext: Vec<RecordExtension<'a>>,
}

impl fmt::Display for Extensions<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            let mut it = self.vext.iter();
            if let Some(ext) = it.next() {
                write!(f, "{:#}", ext)?;
                for ext in it {
                    write!(f, ", {:#}", ext)?;
                }
            }
        } else {
            write!(f, "{:02}", self.vext.len())?;
            for ext in &self.vext {
                write!(f, "{}", ext)?;
            }
        }
        Ok(())
    }
}

fn extension(offset: usize) -> impl for<'a> Fn(&mut &'a [u8]) -> PResult<Extensions<'a>> {
    move |input: &mut &[u8]| {
        (
            n_digits(2),
            repeat(
                1..,
                (n_digits(2), n_digits(2), n_alphanum(3)).map(
                    move |(s, f, tlc): (usize, usize, &[u8])| RecordExtension {
                        start: s - offset - 1,
                        finish: f - offset,
                        tlc,
                        offset,
                    },
                ),
            ),
        )
            .verify(|(nn, vext): &(usize, Vec<RecordExtension>)| *nn == vext.len())
            .map(|(_, vext)| Extensions { vext })
            .parse_next(input)
    }
}

pub fn i_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    delimited(b'I', extension(35), line_ending)
        .map(Record::I)
        .parse_next(input)
}

pub fn j_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    delimited(b'J', extension(7), line_ending)
        .map(Record::J)
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_i_extension() {
        let line = b"I013638FXA\n";
        if let Record::I(ext) = i_record.parse(line).unwrap() {
            assert_eq!(ext.vext.len(), 1);
            assert_eq!(ext.vext[0].tlc, b"FXA");
            assert_eq!(ext.vext[0].start, 0);
            assert_eq!(ext.vext[0].finish, 3);
        } else {
            panic!()
        }
    }

    #[test]
    fn test_parse_multiple_i_extensions() {
        let line = b"I033638FXA3940SIU4143ENL\n";
        if let Record::I(ext) = i_record.parse(line).unwrap() {
            assert_eq!(ext.vext.len(), 3);
            assert_eq!(ext.vext[0].tlc, b"FXA");
            assert_eq!(ext.vext[1].tlc, b"SIU");
            assert_eq!(ext.vext[2].tlc, b"ENL");
        } else {
            panic!()
        }
    }

    #[test]
    fn test_parse_j_extension() {
        let line = b"J010811HDT\n";
        if let Record::J(ext) = j_record.parse(line).unwrap() {
            assert_eq!(ext.vext.len(), 1);
            assert_eq!(ext.vext[0].tlc, b"HDT");
            assert_eq!(ext.vext[0].start, 0);
            assert_eq!(ext.vext[0].finish, 4);
        } else {
            panic!()
        }
    }

    #[test]
    fn test_i_identity() {
        let line = b"I033638FXA3940SIU4143ENL\n";
        if let Record::I(ext) = i_record.parse(line).unwrap() {
            let formatted = format!("{}\n", ext);
            assert_eq!(formatted.as_bytes(), &line[1..]);
        } else {
            panic!()
        };
    }

    #[test]
    fn test_j_identity() {
        let line = b"J010811HDT\n";
        if let Record::J(ext) = j_record.parse(line).unwrap() {
            let formatted = format!("{}\n", ext);
            assert_eq!(formatted.as_bytes(), &line[1..]);
        } else {
            panic!()
        };
    }

    #[test]
    fn test_parse_invalid_count_mismatch() {
        assert!(i_record.parse(b"I023638FXA\n").is_err());
    }
}
