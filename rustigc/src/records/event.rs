//! Event and metadata record parsers
//!
//! This module handles various IGC record types for events, comments, and metadata:
//!
//! - **D-record**: GPS datum and differential GPS information
//!   - Format: D[2-digit qualifier]XXX... (qualifier + free text)
//!   - Specifies GPS datum (e.g., D2100 for WGS84) and differential GPS station
//!
//! - **L-record**: Logbook/comment records
//!   - Format: L[3-letter source code]XXX... (manufacturer code + text)
//!   - Free-form comments, can be manufacturer-specific (e.g., LFLAFLIGHT NOTES)
//!
//! - **G-record**: Security/signature records for flight verification
//!   - Format: GXXX... (cryptographic signature data)
//!   - Used for tamper detection and authenticity verification
//!   - Multi-line security data combining all G-records into signature
//!
//! - **E-record**: Pilot-initiated events with timestamp
//!   - Format: EHHMMSS[3-letter code][text] (timestamp + event type + optional text)
//!   - Event codes like PEV (pilot event), TST (task start), etc.
//!   - Examples: E123456PEV, E154320TSTTAKEOFF
//!
//! - **F-record**: Satellite constellation with timestamp
//!   - Format: FHHMMSS[sat1][sat2]... (timestamp + list of 2-digit satellite IDs)
//!   - Records which satellites were used for position fix
//!   - Example: F123456010305071113 (satellites 01, 03, 05, 07, 11, 13)
//!
//! - **K-record**: Extension data with timestamp
//!   - Format: KHHMMSSXXX... (timestamp + extension data fields)
//!   - Additional sensor data at lower frequency than B-records
//!   - Fields defined by J-record extensions (similar to I-records for B-records)
//!   - Example: K12345612345 with J-record defining field positions

use std::fmt;

use winnow::error::Result as PResult;
use winnow::prelude::*;
use winnow::{
    ascii::{line_ending, till_line_ending},
    combinator::delimited,
};

use super::utils::ts_to_igc;
use super::utils::ts_to_sec;
use super::Record;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TimedEvent<'a> {
    pub timestamp: u32,
    pub text: &'a str,
}

impl<'a> fmt::Display for TimedEvent<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}: {}", self.timestamp, self.text)
        } else {
            write!(f, "{}{}", ts_to_igc(self.timestamp), self.text)
        }
    }
}

fn text_event<'a>() -> impl Fn(&mut &'a str) -> PResult<&'a str> {
    move |input: &mut &str| till_line_ending.parse_next(input)
}

fn timed_event<'a>() -> impl Fn(&mut &'a str) -> PResult<TimedEvent<'a>> {
    move |input: &mut &str| {
        (ts_to_sec, till_line_ending)
            .map(|(timestamp, text): (_, &str)| TimedEvent { timestamp, text })
            .parse_next(input)
    }
}

pub fn d_record<'a>(input: &mut &'a str) -> PResult<Record<'a>> {
    delimited('D', text_event(), line_ending)
        .map(Record::D)
        .parse_next(input)
}

pub fn l_record<'a>(input: &mut &'a str) -> PResult<Record<'a>> {
    delimited('L', text_event(), line_ending)
        .map(Record::L)
        .parse_next(input)
}

pub fn g_record<'a>(input: &mut &'a str) -> PResult<Record<'a>> {
    delimited('G', text_event(), line_ending)
        .map(Record::G)
        .parse_next(input)
}

pub fn e_record<'a>(input: &mut &'a str) -> PResult<Record<'a>> {
    delimited('E', timed_event(), line_ending)
        .map(Record::E)
        .parse_next(input)
}

pub fn f_record<'a>(input: &mut &'a str) -> PResult<Record<'a>> {
    delimited('F', timed_event(), line_ending)
        .map(Record::F)
        .parse_next(input)
}

pub fn k_record<'a>(input: &mut &'a str) -> PResult<Record<'a>> {
    delimited('K', timed_event(), line_ending)
        .map(Record::K)
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_l_record() {
        let line = "LXGD GpsDumpLinux version 0.27\n";
        if let Record::L(text) = l_record.parse(line).unwrap() {
            assert_eq!(text, "XGD GpsDumpLinux version 0.27");
        } else {
            assert!(false)
        }
    }

    #[test]
    fn test_parse_g_record() {
        let line = "G351E2000FC0D9C1B\n";
        if let Record::G(text) = g_record.parse(line).unwrap() {
            assert_eq!(text, "351E2000FC0D9C1B");
        } else {
            assert!(false)
        }
    }

    #[test]
    fn test_parse_e_record() {
        let line = "E101409PEV\n";
        if let Record::E(event) = e_record.parse(line).unwrap() {
            assert_eq!(event.timestamp, 36849);
            assert_eq!(event.text, "PEV");
        } else {
            assert!(false)
        }
    }

    #[test]
    fn test_parse_e_record_with_text() {
        let line = "E114734BFION AH\n";
        if let Record::E(event) = e_record.parse(line).unwrap() {
            assert_eq!(event.timestamp, 42454);
            assert_eq!(event.text, "BFION AH");
        } else {
            assert!(false)
        }
    }

    #[test]
    fn test_parse_f_record() {
        let line = "F09093227163023070801103221\n";
        if let Record::F(event) = f_record.parse(line).unwrap() {
            assert_eq!(event.timestamp, 32972);
            assert_eq!(event.text, "27163023070801103221");
        } else {
            assert!(false)
        }
    }

    #[test]
    fn test_parse_k_record() {
        let line = "K09115208100062\n";
        if let Record::K(event) = k_record.parse(line).unwrap() {
            assert_eq!(event.timestamp, 33112);
            assert_eq!(event.text, "08100062");
        } else {
            assert!(false)
        }
    }

    #[test]
    fn test_l_identity() {
        let line = "LXGD GpsDumpLinux version 0.27\n";
        if let Record::L(text) = l_record.parse(line).unwrap() {
            let formatted = format!("{}\n", text);
            assert_eq!(formatted, &line[1..]);
        } else {
            assert!(false)
        };
    }

    #[test]
    fn test_g_identity() {
        let line = "G351E2000FC0D9C1B\n";
        if let Record::G(text) = g_record.parse(line).unwrap() {
            let formatted = format!("{}\n", text);
            assert_eq!(formatted, &line[1..]);
        } else {
            assert!(false)
        };
    }

    #[test]
    fn test_e_identity() {
        let line = "E101409PEV\n";
        if let Record::E(event) = e_record.parse(line).unwrap() {
            let formatted = format!("{}\n", event);
            assert_eq!(formatted, &line[1..]);
        } else {
            assert!(false)
        };
    }

    #[test]
    fn test_f_identity() {
        let line = "F09093227163023070801103221\n";
        if let Record::F(event) = f_record.parse(line).unwrap() {
            let formatted = format!("{}\n", event);
            assert_eq!(formatted, &line[1..]);
        } else {
            assert!(false)
        };
    }

    #[test]
    fn test_k_identity() {
        let line = "K09115208100062\n";
        if let Record::K(event) = k_record.parse(line).unwrap() {
            let formatted = format!("{}\n", event);
            assert_eq!(formatted, &line[1..]);
        } else {
            assert!(false)
        };
    }

    #[test]
    fn test_parse_invalid_timestamp() {
        assert!(e_record.parse("E999999PEV\n").is_err());
        assert!(f_record.parse("F256030DATA\n").is_err());
        assert!(k_record.parse("K123490DATA\n").is_err());
    }
}
