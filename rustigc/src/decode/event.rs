// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Parsers for the records carrying text, with or without a time. Only the time is decoded; the
//! rest of the line is kept as written.
//!
//! - `D`: GPS datum and differential GPS station, free text after `D`.
//! - `L`: comment, free text after `L`, usually led by a 3-letter source code.
//! - `G`: security signature data.
//! - `E`: pilot event, `EHHMMSS` then a 3-letter code and optional text, e.g. `E154320PEV`.
//! - `F`: satellites in use, `FHHMMSS` then 2-digit satellite ids, e.g. `F123456010305`.
//! - `K`: sensor data, `KHHMMSS` then the fields the J-record lays out.

use std::fmt;

use winnow::combinator::delimited;
use winnow::error::Result as PResult;
use winnow::prelude::*;

use super::Record;
use super::utils::{robust_ending_eof, till_robust_ending};
use super::utils::{ts_to_igc, ts_to_ms};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
/// A record that carries nothing but text: a comment, a signature, a GPS datum.
pub struct TextEvent<'a> {
    /// The line past its record letter, as written.
    pub text: &'a [u8],
}

impl fmt::Display for TextEvent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.text))
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
/// A record that carries text and a time: an event, a satellite list, sensor data.
pub struct TimedEvent<'a> {
    /// Milliseconds from UTC midnight of the flight's first day, carried past 86,400,000.
    pub timestamp: u32,
    /// The line past the timestamp, as written.
    pub text: &'a [u8],
}

impl fmt::Display for TimedEvent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = String::from_utf8_lossy(self.text);
        if f.alternate() {
            write!(f, "{}: {}", self.timestamp, text)
        } else {
            write!(f, "{}{}", ts_to_igc(self.timestamp), text)
        }
    }
}

/// The rest of the line as a [`TextEvent`].
fn text_event<'a>() -> impl Fn(&mut &'a [u8]) -> PResult<TextEvent<'a>> {
    move |input: &mut &[u8]| {
        till_robust_ending
            .map(|text| TextEvent { text })
            .parse_next(input)
    }
}

/// `HHMMSS` then the rest of the line, as a [`TimedEvent`].
fn timed_event<'a>() -> impl Fn(&mut &'a [u8]) -> PResult<TimedEvent<'a>> {
    move |input: &mut &[u8]| {
        (ts_to_ms, till_robust_ending)
            .map(|(timestamp, text): (_, &[u8])| TimedEvent { timestamp, text })
            .parse_next(input)
    }
}

/// Parses one D-record.
pub fn d_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    delimited(b'D', text_event(), robust_ending_eof)
        .map(Record::D)
        .parse_next(input)
}

/// Parses one L-record.
pub fn l_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    delimited(b'L', text_event(), robust_ending_eof)
        .map(Record::L)
        .parse_next(input)
}

/// Parses one G-record.
pub fn g_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    delimited(b'G', text_event(), robust_ending_eof)
        .map(Record::G)
        .parse_next(input)
}

/// Parses one E-record.
pub fn e_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    delimited(b'E', timed_event(), robust_ending_eof)
        .map(Record::E)
        .parse_next(input)
}

/// Parses one F-record.
pub fn f_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    delimited(b'F', timed_event(), robust_ending_eof)
        .map(Record::F)
        .parse_next(input)
}

/// Parses one K-record.
pub fn k_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    delimited(b'K', timed_event(), robust_ending_eof)
        .map(Record::K)
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_l_record() {
        let line = b"LXGD GpsDumpLinux version 0.27\n";
        if let Record::L(event) = l_record.parse(line).unwrap() {
            assert_eq!(event.text, b"XGD GpsDumpLinux version 0.27");
        } else {
            panic!()
        }
    }

    #[test]
    fn test_parse_g_record() {
        let line = b"G351E2000FC0D9C1B\n";
        if let Record::G(event) = g_record.parse(line).unwrap() {
            assert_eq!(event.text, b"351E2000FC0D9C1B");
        } else {
            panic!()
        }
    }

    #[test]
    fn test_parse_e_record() {
        let line = b"E101409PEV\n";
        if let Record::E(event) = e_record.parse(line).unwrap() {
            assert_eq!(event.timestamp, 36_849_000);
            assert_eq!(event.text, b"PEV");
        } else {
            panic!()
        }
    }

    #[test]
    fn test_parse_e_record_with_text() {
        let line = b"E114734BFION AH\n";
        if let Record::E(event) = e_record.parse(line).unwrap() {
            assert_eq!(event.timestamp, 42_454_000);
            assert_eq!(event.text, b"BFION AH");
        } else {
            panic!()
        }
    }

    #[test]
    fn test_parse_f_record() {
        let line = b"F09093227163023070801103221\n";
        if let Record::F(event) = f_record.parse(line).unwrap() {
            assert_eq!(event.timestamp, 32_972_000);
            assert_eq!(event.text, b"27163023070801103221");
        } else {
            panic!()
        }
    }

    #[test]
    fn test_parse_k_record() {
        let line = b"K09115208100062\n";
        if let Record::K(event) = k_record.parse(line).unwrap() {
            assert_eq!(event.timestamp, 33_112_000);
            assert_eq!(event.text, b"08100062");
        } else {
            panic!()
        }
    }

    #[test]
    fn test_l_identity() {
        let line = b"LXGD GpsDumpLinux version 0.27\n";
        if let Record::L(event) = l_record.parse(line).unwrap() {
            let formatted = format!("{}\n", event);
            assert_eq!(formatted.as_bytes(), &line[1..]);
        } else {
            panic!()
        };
    }

    #[test]
    fn test_g_identity() {
        let line = b"G351E2000FC0D9C1B\n";
        if let Record::G(event) = g_record.parse(line).unwrap() {
            let formatted = format!("{}\n", event);
            assert_eq!(formatted.as_bytes(), &line[1..]);
        } else {
            panic!()
        };
    }

    #[test]
    fn test_e_identity() {
        let line = b"E101409PEV\n";
        if let Record::E(event) = e_record.parse(line).unwrap() {
            let formatted = format!("{}\n", event);
            assert_eq!(formatted.as_bytes(), &line[1..]);
        } else {
            panic!()
        };
    }

    #[test]
    fn test_f_identity() {
        let line = b"F09093227163023070801103221\n";
        if let Record::F(event) = f_record.parse(line).unwrap() {
            let formatted = format!("{}\n", event);
            assert_eq!(formatted.as_bytes(), &line[1..]);
        } else {
            panic!()
        };
    }

    #[test]
    fn test_k_identity() {
        let line = b"K09115208100062\n";
        if let Record::K(event) = k_record.parse(line).unwrap() {
            let formatted = format!("{}\n", event);
            assert_eq!(formatted.as_bytes(), &line[1..]);
        } else {
            panic!()
        };
    }

    #[test]
    fn test_parse_invalid_timestamp() {
        assert!(e_record.parse(b"E999999PEV\n").is_err());
        assert!(f_record.parse(b"F256030DATA\n").is_err());
        assert!(k_record.parse(b"K123490DATA\n").is_err());
    }
}
