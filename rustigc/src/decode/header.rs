// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! H-record (header/metadata) parser
//!
//! General format: `H[F/O/P][XXX][:][text]`
//! - `F`/`O`/`P`: entered by the flight recorder, an official observer, or the pilot
//! - `XXX`: 3-letter code
//! - optional colon, then the value

use std::fmt;

use winnow::ascii::alphanumeric1;
use winnow::combinator::alt;
use winnow::combinator::{delimited, opt};
use winnow::error::Result as PResult;
use winnow::prelude::*;
use winnow::token::take;

use super::utils::{n_alphanum, robust_ending_eof, till_robust_ending};
use super::Record;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Deserialize, Serialize),
    serde(rename_all = "lowercase")
)]
/// Who entered a header.
pub enum HeaderOrigin {
    /// Written by the recorder itself.
    FlightRecorder,
    /// Entered by an official observer.
    Observer,
    /// Entered by the pilot.
    Pilot,
    /// Unexpected origin.
    Unknown,
}

impl HeaderOrigin {
    const fn as_byte(&self) -> u8 {
        match self {
            HeaderOrigin::FlightRecorder => b'F',
            HeaderOrigin::Observer => b'O',
            HeaderOrigin::Pilot => b'P',
            HeaderOrigin::Unknown => b'U',
        }
    }

    /// Human-readable origin, `"Flight Recorder"`, `"Observer"`, `"Pilot"` or `"Unknown"`.
    pub const fn as_str(&self) -> &str {
        match self {
            HeaderOrigin::FlightRecorder => "Flight Recorder",
            HeaderOrigin::Observer => "Observer",
            HeaderOrigin::Pilot => "Pilot",
            HeaderOrigin::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for HeaderOrigin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.as_str())
        } else {
            write!(f, "{}", self.as_byte() as char)
        }
    }
}

// Metadata is the kind of things we will eventually provide
// and their number is negligible so we are better allocating
// the strings directly

/// The value of one header.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct HeaderData {
    /// The value as written, trimmed of the key and its separator.
    pub text: String,
    /// Header origin
    pub origin: HeaderOrigin,
}

/// One header, as a key and its value (H-record).
///
/// [`Log`] keeps these as a map instead; this is the record-level form.
///
/// [`Log`]: crate::Log
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Header {
    /// Three-letter code: `"PLT"` for the pilot, `"GTY"` for the glider, `"DTE"` for the date, ...
    pub key: String,
    /// What the header holds.
    pub value: HeaderData,
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(
                f,
                "{:#} {}: {}",
                self.value.origin, self.key, self.value.text
            )
        } else {
            write!(f, "{}{}{}", self.value.origin, self.key, self.value.text)
        }
    }
}

impl From<Header> for Record<'_> {
    fn from(v: Header) -> Self {
        Record::H(v)
    }
}

fn horigin(input: &mut &[u8]) -> PResult<HeaderOrigin> {
    alt((
        (HeaderOrigin::FlightRecorder.as_byte()).value(HeaderOrigin::FlightRecorder),
        (HeaderOrigin::Observer.as_byte()).value(HeaderOrigin::Observer),
        (HeaderOrigin::Pilot.as_byte()).value(HeaderOrigin::Pilot),
        (take(1usize)).value(HeaderOrigin::Unknown), // Match creative origins
    ))
    .parse_next(input)
}

fn hkey<'a>(input: &mut &'a [u8]) -> PResult<&'a [u8]> {
    (n_alphanum(3), opt((alphanumeric1, (b':'))))
        .map(|(m, _): (&[u8], _)| m)
        .parse_next(input)
}

/// Parses one H-record into a [`Header`], its key ready to index the log's header map.
pub fn h_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    delimited(b'H', (horigin, hkey, till_robust_ending), robust_ending_eof)
        .map(|(origin, key, v)| {
            let key = String::from_utf8_lossy(key).to_string();
            let text = String::from_utf8_lossy(v).trim().to_string();
            Header {
                key,
                value: HeaderData { text, origin },
            }
            .into()
        })
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_h_record() {
        let line = b"HFDTE150120\n";
        if let Record::H(inner) = h_record.parse(line).unwrap() {
            assert_eq!(inner.key, "DTE");
            assert_eq!(inner.value.text, "150120");
            assert_eq!(inner.value.origin, HeaderOrigin::FlightRecorder);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_parse_valid_longh_record() {
        let line = b"HFPLTPILOTINCHARGE:Tripoux Robert\n";
        if let Record::H(inner) = h_record.parse(line).unwrap() {
            assert_eq!(inner.key, "PLT");
            assert_eq!(inner.value.text, "Tripoux Robert");
            assert_eq!(inner.value.origin, HeaderOrigin::FlightRecorder);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_parse_invalid_origin() {
        let line = b"HXDTE150120\n";
        if let Record::H(inner) = h_record.parse(line).unwrap() {
            assert_eq!(inner.value.origin, HeaderOrigin::Unknown);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_parse_invalid_key_non_alphanum() {
        assert!(h_record.parse(b"HFD  150120\n").is_err());
    }

    #[test]
    fn test_identity() {
        let line = b"HFDTE150120\n";
        if let Record::H(inner) = h_record.parse(line).unwrap() {
            let formatted = format!("{}\n", inner);
            assert_eq!(formatted.as_bytes(), &line[1..]);
        } else {
            panic!()
        };
    }
}
