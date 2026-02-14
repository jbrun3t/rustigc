//! C-record (task declaration) parser
//!
//! C-records define the declared task for a flight. Unlike other records, C-records
//! span multiple lines to represent the complete task declaration.
//!
//! Format:
//! - Header line: CDDMMYYHHMMSSDDMMYY<4-digit-obsolete><nn><text>
//!   - C: Record type
//!   - DDMMYYHHMMSS: Declaration timestamp (12 digits)
//!   - DDMMYY: Flight date (6 digits)
//!   - 4 digits: Obsolete field (ignored)
//!   - nn: Number of turnpoints (2 digits, excludes start/finish but spec is ambiguous)
//!   - text: Task description
//!
//! - Turnpoint lines (one per turnpoint): CDDMMmmmN/SDDDMMmmmE/W<text>
//!   - C: Record type
//!   - DDMMmmmN/S: Latitude
//!   - DDDMMmmmE/W: Longitude
//!   - text: Turnpoint name/description
//!
//! Example:
//! ```text
//! C15062411010115062400000203Task description
//! C5230000N00030000WTakeoff
//! C5240000N00040000WTP1
//! C5250000N00050000WTP2
//! C5230000N00030000WLanding
//! ```

use std::fmt;

use winnow::combinator::repeat;
use winnow::error::Result as PResult;
use winnow::prelude::*;
use winnow::stream::AsChar;
use winnow::token::{take, take_while};
use winnow::{
    ascii::{line_ending, till_line_ending},
    combinator::delimited,
};

use super::utils::{latitude, longitude};
use super::utils::{latitude_to_igc, longitude_to_igc};
use super::Record;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Declared task turnpoint
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TurnPoint {
    pub lat: f64,
    pub lon: f64,
    pub text: String,
}

impl fmt::Display for TurnPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, " TP - [{},{}]: {}", self.lat, self.lon, self.text)
        } else {
            write!(
                f,
                "C{}{}{}",
                latitude_to_igc(self.lat),
                longitude_to_igc(self.lon),
                self.text
            )
        }
    }
}

/// Declared task
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Task {
    pub declaration: String, // As DDMMYYHHMMSS
    pub flight: String,      // As DDMMYY
    pub text: String,
    pub turnpoints: Vec<TurnPoint>,
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(
                f,
                "{} TP - {} - {} - {}",
                self.turnpoints.len(),
                self.declaration,
                self.flight,
                self.text
            )?;
        } else {
            write!(
                f,
                "{}{}0000{:02}{}",
                self.declaration,
                self.flight,
                if self.turnpoints.len() < 2 {
                    0
                } else {
                    self.turnpoints.len() - 2
                },
                self.text
            )?;
        }

        for tp in &self.turnpoints {
            writeln!(f)?;
            tp.fmt(f)?
        }
        Ok(())
    }
}

impl<'a> From<Task> for Record<'a> {
    fn from(v: Task) -> Self {
        Record::C(v)
    }
}

pub fn turnpoint(input: &mut &[u8]) -> PResult<TurnPoint> {
    delimited(b'C', (latitude, longitude, till_line_ending), line_ending)
        .map(|(ns, ew, t): (_, _, &[u8])| TurnPoint {
            lat: (ns as f64) / 60000.0,
            lon: (ew as f64) / 60000.0,
            text: std::str::from_utf8(t).unwrap().trim().to_string(),
        })
        .parse_next(input)
}

// NOTE: This record does not behave like the other.
// I will consume more than one line. It assumes that all C-Records are
// following each other which makes sense and seems to be the case. It
// allows to construct the Task directly which makes the parsing a lot
// simpler
//
// NOTE #2: Do not bother validating the number of turnpoints declared.
// The spec is not clear on that point. It says the number exclude
// start/finish, says nothing nothing about takeoff/landing or how
// the parser is supposed to recognize those TP. In the end, the
// the number of TP found could be NN + 2, or NN + 4 depending on
// the FR and Task entered. The info is not needed actually, just
// ignore it
pub fn c_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    (
        delimited(
            b'C',
            (
                take_while(12..=12, AsChar::is_dec_digit),
                take_while(6..=6, AsChar::is_dec_digit),
                take(4usize), // Obsolete field
                take(2usize),
                till_line_ending,
            ),
            line_ending,
        ),
        repeat(1.., turnpoint),
    )
        .map(
            |((d, f, _, _, t), turnpoints): ((&[u8], &[u8], _, _, &[u8]), _)| {
                Task {
                    declaration: std::str::from_utf8(d).unwrap().to_string(),
                    flight: std::str::from_utf8(f).unwrap().to_string(),
                    text: std::str::from_utf8(t).unwrap().trim().to_string(),
                    turnpoints,
                }
                .into()
            },
        )
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_task_real_example() {
        // Coordinates chosen for exact floating point representation
        let input = b"C050822090459000000000502\n\
                      C0000000N00000000E\n\
                      C5130000N00130000WLasham Finish North\n\
                      C5204577N00307663WHay-on-wye\n\
                      C5258317N00003531WBoston\n\
                      C5130000N00130000WLasham Finish North\n\
                      C0000000N00000000E\n";

        if let Record::C(task) = c_record.parse(input).unwrap() {
            assert_eq!(task.declaration, "050822090459");
            assert_eq!(task.flight, "000000");
            assert_eq!(task.text, "");
            assert_eq!(task.turnpoints.len(), 6);

            assert_eq!(task.turnpoints[0].text, "");
            assert_eq!(task.turnpoints[0].lat, 0.0);
            assert_eq!(task.turnpoints[0].lon, 0.0);

            assert_eq!(task.turnpoints[1].text, "Lasham Finish North");
            assert_eq!(task.turnpoints[1].lat, 51.5);
            assert_eq!(task.turnpoints[1].lon, -1.5);

            assert_eq!(task.turnpoints[4].text, "Lasham Finish North");
            assert_eq!(task.turnpoints[4].lat, 51.5);
            assert_eq!(task.turnpoints[4].lon, -1.5);
        } else {
            panic!("Expected Record::C");
        }
    }

    #[test]
    fn test_parse_turnpoint() {
        let input = b"C5200000N00130000WTurnpoint Name\n";
        let tp = turnpoint.parse(input).unwrap();

        assert_eq!(tp.lat, 52.0);
        assert_eq!(tp.lon, -1.5);
        assert_eq!(tp.text, "Turnpoint Name");
    }

    #[test]
    fn test_parse_bad_turnpoint_invalid_lat() {
        let input = b"C9506343N00006198WBad latitude\n";
        assert!(turnpoint.parse(input).is_err());
    }

    #[test]
    fn test_parse_bad_header_too_short() {
        let input = b"C05082209045900000\n";
        assert!(c_record.parse(input).is_err());
    }

    #[test]
    fn test_parse_bad_header_no_turnpoints() {
        let input = b"C050822090459000000000502Task without turnpoints\n";
        assert!(c_record.parse(input).is_err());
    }
}
