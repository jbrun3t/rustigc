// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! I- and J-record parsers: the extension fields appended to every B- or K-record.
//!
//! Format `Inn` then, per field, `SSFFCCC`: 1-based inclusive start and end bytes and a 3-letter
//! code. `I033638FXA3940SIU4143ENL` puts FXA on bytes 36–38, SIU on 39–40 and ENL on 41–43.
//! Positions are stored 0-based and end-exclusive within the extension area, so `start..finish`
//! slices it directly.

use std::fmt;

use winnow::combinator::{delimited, repeat};
use winnow::error::Result as PResult;
use winnow::prelude::*;

use super::Record;
use super::utils::{n_alphanum, n_digits, robust_ending_eof};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(bound(deserialize = "'de: 'a")))]
/// One extension field, as an I or J record defines it.
pub struct RecordExtension<'a> {
    /// First byte, 0-based within the extension area.
    pub start: usize,
    /// End byte, exclusive.
    pub finish: usize,
    /// Three-letter code, e.g. `FXA`, `SIU`, `ENL`.
    pub tlc: &'a [u8],
    /// Length of the owning record's fixed part, 35 for B and 7 for K; `Display` adds it back to
    /// print the file's positions.
    pub offset: usize,
}

impl fmt::Display for RecordExtension<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tlc = String::from_utf8_lossy(self.tlc);
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
/// The extension fields an I or J record defines, in the order they appear.
pub struct Extensions<'a> {
    /// The fields, each slicing its own span out of the record it extends.
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

/// `nn` then the field definitions. Refused when the count disagrees with the fields, or a field
/// starts inside the fixed part or ends before it starts.
fn extension(offset: usize) -> impl for<'a> Fn(&mut &'a [u8]) -> PResult<Extensions<'a>> {
    move |input: &mut &[u8]| {
        (
            n_digits(2),
            repeat(
                0..,
                (n_digits(2), n_digits(2), n_alphanum(3))
                    .verify(move |&(s, f, _): &(usize, usize, &[u8])| {
                        s > offset && f >= s
                    })
                    .map(move |(s, f, tlc): (usize, usize, &[u8])| RecordExtension {
                        start: s - offset - 1,
                        finish: f - offset,
                        tlc,
                        offset,
                    }),
            ),
        )
            .verify(|(nn, vext): &(usize, Vec<RecordExtension>)| *nn == vext.len())
            .map(|(_, vext)| Extensions { vext })
            .parse_next(input)
    }
}

/// Parses one I-record, the fields extending every B-record.
pub fn i_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    delimited(b'I', extension(35), robust_ending_eof)
        .map(Record::I)
        .parse_next(input)
}

/// Parses one J-record, the fields extending every K-record.
pub fn j_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    delimited(b'J', extension(7), robust_ending_eof)
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

    /// Some recorders write an I-record with no field.
    #[test]
    fn test_i_zero() {
        let line = b"I00\n";
        if let Record::I(ext) = i_record.parse(line).unwrap() {
            assert!(ext.vext.is_empty());
        } else {
            panic!()
        }
    }

    #[test]
    fn test_parse_invalid_count_mismatch() {
        assert!(i_record.parse(b"I023638FXA\n").is_err());
    }

    #[test]
    fn test_j_start_below_offset() {
        assert!(j_record.parse(b"J010102HDT\n").is_err());
    }

    #[test]
    fn test_i_finish_before_start() {
        assert!(i_record.parse(b"I013836FXA\n").is_err());
    }
}
