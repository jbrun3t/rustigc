//! H-record (header/metadata) parser
//!
//! H-records contain flight metadata in various formats:
//! - HFDTE: Date (DDMMYY)
//! - HFPLT + PILOT: Pilot name
//! - HFGTY + GLIDERTYPE: Glider type
//! - HFGID + GLIDERID: Glider ID
//! - HFSTA + SITE: Takeoff site
//!
//! General format: H[F/O][XXX][colon][text]
//! - H: Record type
//! - F: Fixed data / P: Pilot entered / O: Official observer
//! - XXX: 3-letter code
//! - Optional colon separator and value

use std::fmt;

use winnow::ascii::alphanumeric1;
use winnow::combinator::alt;
use winnow::combinator::{delimited, opt};
use winnow::error::Result as PResult;
use winnow::prelude::*;

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
pub enum HeaderOrigin {
    FlightRecorder,
    Observer,
    Pilot,
}

impl HeaderOrigin {
    const fn as_byte(&self) -> u8 {
        match self {
            HeaderOrigin::FlightRecorder => b'F',
            HeaderOrigin::Observer => b'O',
            HeaderOrigin::Pilot => b'P',
        }
    }

    pub const fn as_str(&self) -> &str {
        match self {
            HeaderOrigin::FlightRecorder => "Flight Recorder",
            HeaderOrigin::Observer => "Observer",
            HeaderOrigin::Pilot => "Pilot",
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

/// Single header content entry
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct HeaderData {
    pub text: String,
    pub origin: HeaderOrigin,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Header {
    pub key: String,
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
    ))
    .parse_next(input)
}

fn hkey<'a>(input: &mut &'a [u8]) -> PResult<&'a [u8]> {
    (n_alphanum(3), opt((alphanumeric1, (b':'))))
        .map(|(m, _): (&[u8], _)| m)
        .parse_next(input)
}

/// Provide a tuple with ( HeaderID, HeaderData ) ready for insertion in hashmap
pub fn h_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    delimited(b'H', (horigin, hkey, till_robust_ending), robust_ending_eof)
        .map(|(origin, key, v)| {
            let key: String = std::str::from_utf8(key).unwrap().to_string();
            Header {
                key,
                value: HeaderData {
                    text: std::str::from_utf8(v).unwrap().to_string(),
                    origin,
                },
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
        assert!(h_record.parse(b"HXDTE150120\n").is_err());
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
