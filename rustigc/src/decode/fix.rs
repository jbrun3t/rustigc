// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

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

use winnow::combinator::alt;
use winnow::combinator::delimited;
use winnow::error::Result as PResult;
use winnow::prelude::*;

use super::utils::robust_ending_eof;
use super::utils::till_robust_ending;
use super::utils::{latitude, longitude, n_digits, ts_to_ms};
use super::utils::{latitude_to_igc, longitude_to_igc, ts_to_igc};
use super::Record;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One position fix.
///
/// The unit of a track. A timestamp counts milliseconds from the origin [`Log::datetime`] states
/// and carries no date of its own.
///
/// `repr(C)`, 32 bytes: the `u32` timestamp, four bytes of alignment padding, then the `f64` and
/// `i32` fields. A binding reading a track as raw bytes must account for that padding. Do not
/// reorder the fields !
///
/// [`Log::datetime`]: crate::Log::datetime
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Fix {
    /// Milliseconds since UTC midnight of the flight's first day. A flight crossing midnight
    /// keeps counting past 86,400,000, so this can exceed one day.
    pub timestamp: u32,
    /// Latitude in decimal degrees, north positive.
    pub lat: f64,
    /// Longitude in decimal degrees, east positive.
    pub lon: f64,
    /// Pressure altitude in meters.
    pub baro_alt: i32,
    /// GNSS (GPS) altitude in meters.
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

impl crate::utils::geometry::PointCoords<f64> for Fix {
    #[inline]
    fn x(&self) -> f64 {
        self.lon
    }

    #[inline]
    fn y(&self) -> f64 {
        self.lat
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
/// A fix as its B-record holds it.
///
/// [`Log`]: crate::Log
pub struct RawFix<'a> {
    /// The position itself.
    pub fix: Fix,
    /// Whether the recorder called the fix a valid 3D one.
    pub valid: bool,
    /// Extension bytes past the fixed part, laid out by the file's I-record.
    pub ext: &'a [u8],
}

impl fmt::Display for RawFix<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ext = String::from_utf8_lossy(self.ext);
        if f.alternate() {
            write!(
                f,
                "{} - {} - {}",
                if self.valid { "OK" } else { "KO" },
                self.fix,
                if self.ext.len() == 2 { "none" } else { &ext }
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
                ext
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

fn altitude(input: &mut &[u8]) -> PResult<i32> {
    n_digits(5).parse_next(input)
}

fn valid3d(input: &mut &[u8]) -> PResult<bool> {
    alt(((b'A').value(true), (b'V').value(false))).parse_next(input)
}

fn fix(input: &mut &[u8]) -> PResult<(Fix, bool)> {
    (ts_to_ms, latitude, longitude, valid3d, altitude, altitude)
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

pub fn b_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    delimited(b'B', (fix, till_robust_ending), robust_ending_eof)
        .map(|((fix, valid), ext)| RawFix { fix, valid, ext }.into())
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_b_record() {
        let line = b"B1101355206300N00006180WA0058700558\n";
        if let Record::B(rec) = b_record.parse(line).unwrap() {
            assert_eq!(rec.fix.timestamp, 39_695_000);
            assert_eq!(rec.fix.lat, 52.105);
            assert_eq!(rec.fix.lon, -0.103);
            assert_eq!(rec.fix.baro_alt, 587);
            assert_eq!(rec.fix.gnss_alt, 558);
            assert!(rec.valid);
        } else {
            panic!()
        };
    }

    #[test]
    fn test_parse_invalid_fix() {
        let line = b"B1200005012345N12034567WV0100001000\n";
        if let Record::B(rec) = b_record.parse(line).unwrap() {
            assert!(!rec.valid);
        } else {
            panic!()
        };
    }

    #[test]
    fn test_parse_invalid_line_too_short() {
        assert!(b_record.parse(b"B110135\n").is_err());
    }

    #[test]
    fn test_parse_southern_eastern_negative_alt() {
        let line = b"B1200003000000S12000000EA-0100-0200\n";
        if let Record::B(rec) = b_record.parse(line).unwrap() {
            assert_eq!(rec.fix.timestamp, 43_200_000);
            assert_eq!(rec.fix.lat, -30.0);
            assert_eq!(rec.fix.lon, 120.0);
            assert_eq!(rec.fix.baro_alt, -100);
            assert_eq!(rec.fix.gnss_alt, -200);
            assert!(rec.valid);
        } else {
            panic!()
        };
    }

    #[test]
    fn test_parse_with_extensions() {
        let line = b"B1200005012345N00012345WA00500005001234567890\n";
        if let Record::B(rec) = b_record.parse(line).unwrap() {
            assert_eq!(rec.ext, b"1234567890");
            assert_eq!(rec.fix.timestamp, 43_200_000);
        } else {
            panic!()
        };
    }

    #[test]
    fn test_identity() {
        let line = b"B1200003000000N12000000EA0050000500\n";
        if let Record::B(rec) = b_record.parse(line).unwrap() {
            let formatted = format!("{}\n", rec);
            assert_eq!(formatted.as_bytes(), &line[1..]);
        } else {
            panic!()
        };
    }

    #[test]
    fn test_parse_many_cr() {
        let line = b"B1639004549904N00256219EA0245602531 \r\r\n";
        if let Record::B(rec) = b_record.parse(line).unwrap() {
            assert!(rec.valid);
        } else {
            panic!()
        };
    }
}
