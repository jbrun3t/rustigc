// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! C-record (task declaration) parser.
//!
//! A task spans consecutive C-records: an optional declaration line, then one line per turnpoint.
//! - Declaration: `CDDMMYYHHMMSSDDMMYYnnnnTTtext`, the declaration time, the flight date, an
//!   obsolete 4-digit field, the turnpoint count and a description.
//! - Turnpoint: `CDDMMmmmNDDDMMmmmEtext`, latitude, longitude and name.
//!
//! Example:
//! ```text
//! C150624110101150624000002Task description
//! C5230000N00030000WTakeoff
//! C5240000N00040000WTP1
//! C5250000N00050000WTP2
//! C5230000N00030000WLanding
//! ```

use std::fmt;

use winnow::combinator::delimited;
use winnow::combinator::{opt, repeat};
use winnow::error::Result as PResult;
use winnow::prelude::*;
use winnow::stream::AsChar;
use winnow::token::{take, take_while};

use super::Record;
use super::utils::{latitude, longitude, robust_ending_eof, till_robust_ending};
use super::utils::{latitude_to_igc, longitude_to_igc};

use serde::{Deserialize, Serialize};

/// One turnpoint of a declared task.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TurnPoint {
    /// Latitude in decimal degrees, north positive.
    pub lat: f64,
    /// Longitude in decimal degrees, east positive.
    pub lon: f64,
    /// Name the pilot gave it.
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

/// When a task was declared and what for.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Declaration {
    /// When the declaration was made, as written: `DDMMYYHHMMSS`.
    pub date: String,
    /// Day the task is flown, as written: `DDMMYY`.
    pub flight: String,
    /// Free-form task description.
    pub text: String,
}

/// The task the pilot declared before flying (C-records).
///
/// Scoring does not use it: [`Log::score`] searches the track as flown.
///
/// [`Log::score`]: crate::Log::score
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Task {
    /// The declaration header, absent when the file jumps straight to turnpoints.
    pub declaration: Option<Declaration>,
    /// Turnpoints in order, usually takeoff and landing included.
    pub turnpoints: Vec<TurnPoint>,
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(d) = &self.declaration {
            if f.alternate() {
                write!(
                    f,
                    "{} TP - {} - {} - {}",
                    self.turnpoints.len(),
                    d.date,
                    d.flight,
                    d.text
                )?;
            } else {
                write!(
                    f,
                    "{}{}0000{:02}{}",
                    d.date,
                    d.flight,
                    if self.turnpoints.len() < 2 {
                        0
                    } else {
                        self.turnpoints.len() - 2
                    },
                    d.text
                )?;
            }
        }

        for tp in &self.turnpoints {
            writeln!(f)?;
            tp.fmt(f)?
        }
        Ok(())
    }
}

impl From<Task> for Record<'_> {
    fn from(v: Task) -> Self {
        Record::C(v)
    }
}

/// Parses one turnpoint line.
pub fn turnpoint(input: &mut &[u8]) -> PResult<TurnPoint> {
    delimited(
        b'C',
        (latitude, longitude, till_robust_ending),
        robust_ending_eof,
    )
    .map(|(ns, ew, t): (_, _, &[u8])| TurnPoint {
        lat: (ns as f64) / 60000.0,
        lon: (ew as f64) / 60000.0,
        text: String::from_utf8_lossy(t).into_owned(),
    })
    .parse_next(input)
}

/// Parses the declaration line; the obsolete field and the turnpoint count are skipped.
pub fn task_declaration(input: &mut &[u8]) -> PResult<Declaration> {
    delimited(
        b'C',
        (
            take_while(12..=12, AsChar::is_dec_digit),
            take_while(6..=6, AsChar::is_dec_digit),
            take(4usize), // Obsolete field
            take(2usize),
            till_robust_ending,
        ),
        robust_ending_eof,
    )
    .map(|(d, f, _, _, t): (&[u8], &[u8], _, _, &[u8])| Declaration {
        date: String::from_utf8_lossy(d).into_owned(),
        flight: String::from_utf8_lossy(f).into_owned(),
        text: String::from_utf8_lossy(t).into_owned(),
    })
    .parse_next(input)
}

/// Parses a whole task, consuming every consecutive C-record: an optional declaration, then the
/// turnpoints.
///
/// The declaration is optional because many recorders omit it. Its turnpoint count is not
/// checked: the spec excludes start and finish from it but says nothing of takeoff and landing, and
/// recorders disagree.
pub fn c_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    (opt(task_declaration), repeat(1.., turnpoint))
        .map(|(declaration, turnpoints)| {
            Task {
                declaration,
                turnpoints,
            }
            .into()
        })
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
            let d = task.declaration.unwrap();
            assert_eq!(d.date, "050822090459");
            assert_eq!(d.flight, "000000");
            assert_eq!(d.text, "");
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
