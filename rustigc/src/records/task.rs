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
