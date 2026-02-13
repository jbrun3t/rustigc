//! B-record (position fix) parser
//!
//! B-record format: BHHMMSSsDDMMmmmNDDDMMmmmEVPPPPPGGGGG
//! - B: Record type
//! - HHMMSS: UTC time
//! - DDMMmmmN/S: Latitude (degrees, minutes with 3 decimals, hemisphere)
//! - DDDMMmmmE/W: Longitude (degrees, minutes with 3 decimals, hemisphere)
//! - V: Validity (A=valid 3D, V=invalid)
//! - PPPPP: Pressure altitude (meters)
//! - GGGGG: GNSS altitude (meters)

use std::fmt;

use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::alt;
use winnow::combinator::delimited;
use winnow::error::Result as PResult;
use winnow::prelude::*;

use super::utils::{latitude, longitude, n_digits, ts_to_sec};
use super::utils::{latitude_to_igc, longitude_to_igc, ts_to_igc};
use super::Record;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Single position fix from B-record
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Fix {
    /// UTC Unix timestamp - Beware of changing days in flight
    pub timestamp: u32,
    /// Point Latitude
    pub lat: f64,
    /// Point Longitude
    pub lon: f64,
    /// Pressure altitude in meters
    pub baro_alt: i32,
    /// GNSS (GPS) altitude in meters
    pub gnss_alt: i32,
}

impl fmt::Display for Fix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: [{:.04},{:.04}] - {} - {}",
            self.timestamp, self.lat, self.lon, self.baro_alt, self.gnss_alt
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct RawFix<'a> {
    pub fix: Fix,
    pub valid: bool,
    pub ext: &'a str,
}

impl<'a> fmt::Display for RawFix<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(
                f,
                "{} - {} - {}",
                if self.valid { "OK" } else { "KO" },
                self.fix,
                if self.ext.len() == 2 {
                    "none"
                } else {
                    self.ext
                }
            )?;
        } else {
            write!(
                f,
                "{}{}{}{}{:05}{:05}{}",
                ts_to_igc(self.fix.timestamp),
                latitude_to_igc(self.fix.lat),
                longitude_to_igc(self.fix.lon),
                if self.valid { 'A' } else { 'V' },
                self.fix.baro_alt,
                self.fix.gnss_alt,
                self.ext
            )?;
        }
        Ok(())
    }
}

impl<'a> From<RawFix<'a>> for Record<'a> {
    fn from(v: RawFix<'a>) -> Self {
        Record::B(v)
    }
}

fn altitude(input: &mut &str) -> PResult<i32> {
    n_digits(5).parse_next(input)
}

fn valid3d(input: &mut &str) -> PResult<bool> {
    alt((('A').value(true), ('V').value(false))).parse_next(input)
}

fn fix(input: &mut &str) -> PResult<(Fix, bool)> {
    (ts_to_sec, latitude, longitude, valid3d, altitude, altitude)
        .map(|(t, ns, ew, v, baro_alt, gnss_alt)| {
            (
                Fix {
                    timestamp: t,
                    lat: (ns as f64) / 60000.0,
                    lon: (ew as f64) / 60000.0,
                    baro_alt,
                    gnss_alt,
                },
                v,
            )
        })
        .parse_next(input)
}

pub fn b_record<'a>(input: &mut &'a str) -> PResult<Record<'a>> {
    delimited('B', (fix, till_line_ending), line_ending)
        .map(|((fix, valid), ext)| RawFix { fix, valid, ext }.into())
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_b_record() {
        let line = "B1101355206300N00006180WA0058700558\n";
        if let Record::B(rec) = b_record.parse(line).unwrap() {
            assert_eq!(rec.fix.timestamp, 39695);
            assert_eq!(rec.fix.lat, 52.105);
            assert_eq!(rec.fix.lon, -0.103);
            assert_eq!(rec.fix.baro_alt, 587);
            assert_eq!(rec.fix.gnss_alt, 558);
            assert!(rec.valid);
        } else {
            assert!(false)
        };
    }

    #[test]
    fn test_parse_invalid_fix() {
        let line = "B1200005012345N12034567WV0100001000\n";
        if let Record::B(rec) = b_record.parse(line).unwrap() {
            assert!(!rec.valid);
        } else {
            assert!(false)
        };
    }

    #[test]
    fn test_parse_invalid_line_too_short() {
        assert!(b_record.parse("B110135\n").is_err());
    }

    #[test]
    fn test_parse_southern_eastern_negative_alt() {
        let line = "B1200003000000S12000000EA-0100-0200\n";
        if let Record::B(rec) = b_record.parse(line).unwrap() {
            assert_eq!(rec.fix.timestamp, 43200);
            assert_eq!(rec.fix.lat, -30.0);
            assert_eq!(rec.fix.lon, 120.0);
            assert_eq!(rec.fix.baro_alt, -100);
            assert_eq!(rec.fix.gnss_alt, -200);
            assert!(rec.valid);
        } else {
            assert!(false)
        };
    }

    #[test]
    fn test_parse_with_extensions() {
        let line = "B1200005012345N00012345WA00500005001234567890\n";
        if let Record::B(rec) = b_record.parse(line).unwrap() {
            assert_eq!(rec.ext, "1234567890");
            assert_eq!(rec.fix.timestamp, 43200);
        } else {
            assert!(false)
        };
    }

    #[test]
    fn test_identity() {
        let line = "B1200003000000N12000000EA0050000500\n";
        if let Record::B(rec) = b_record.parse(line).unwrap() {
            let formatted = format!("{}\n", rec);
            assert_eq!(formatted, &line[1..]);
        } else {
            assert!(false)
        };
    }
}
