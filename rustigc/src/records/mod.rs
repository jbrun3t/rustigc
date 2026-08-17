//! IGC record parsers
//!
//! This module contains parsers for all IGC record types (A-L).

use std::fmt;

pub mod bad;
pub mod event;
pub mod extension;
pub mod fix;
pub mod header;
pub mod recorder;
pub mod task;

mod datetime;
mod utils;

pub use bad::*;
pub use event::*;
pub use extension::*;
pub use fix::*;
pub use header::*;
pub use recorder::*;
pub use task::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(bound(deserialize = "'de: 'a")))]
pub enum Record<'a> {
    A(Recorder),
    B(RawFix<'a>),
    C(Task),
    D(TextEvent<'a>),
    E(TimedEvent<'a>),
    F(TimedEvent<'a>),
    G(TextEvent<'a>),
    H(Header),
    I(Extensions<'a>),
    J(Extensions<'a>),
    K(TimedEvent<'a>),
    L(TextEvent<'a>),
    Bad(&'a [u8]),
}

impl fmt::Display for Record<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            match self {
                Record::A(inner) => write!(f, "Recorder {:#}", inner)?,
                Record::B(inner) => write!(f, "Fix {:#}", inner)?,
                Record::C(inner) => write!(f, "Task {:#}", inner)?,
                Record::D(inner) => write!(f, "DGPS {:#}", inner)?,
                Record::E(inner) => write!(f, "Event {:#}", inner)?,
                Record::F(inner) => write!(f, "Satellite {:#}", inner)?,
                Record::G(inner) => write!(f, "Security {:#}", inner)?,
                Record::H(inner) => write!(f, "Header {:#}", inner)?,
                Record::I(inner) => write!(f, "Fix Extensions {:#}", inner)?,
                Record::J(inner) => write!(f, "Data Extensions {:#}", inner)?,
                Record::K(inner) => write!(f, "Data {:#}", inner)?,
                Record::L(inner) => write!(f, "Comment {:#}", inner)?,
                Record::Bad(inner) => write!(
                    f,
                    "Invalid Record: {}",
                    std::str::from_utf8(inner).unwrap_or("Non ASCII")
                )?,
            }
        } else {
            match self {
                Record::A(inner) => write!(f, "A{}", inner)?,
                Record::B(inner) => write!(f, "B{}", inner)?,
                Record::C(inner) => write!(f, "C{}", inner)?,
                Record::D(inner) => write!(f, "D{}", inner)?,
                Record::E(inner) => write!(f, "E{}", inner)?,
                Record::F(inner) => write!(f, "F{}", inner)?,
                Record::G(inner) => write!(f, "G{}", inner)?,
                Record::H(inner) => write!(f, "H{}", inner)?,
                Record::I(inner) => write!(f, "I{}", inner)?,
                Record::J(inner) => write!(f, "J{}", inner)?,
                Record::K(inner) => write!(f, "K{}", inner)?,
                Record::L(inner) => write!(f, "L{}", inner)?,
                Record::Bad(_inner) => write!(f, "")?,
            }
        }
        Ok(())
    }
}

fn ts_offset(ts: &mut u32, offset: &mut u32, last: &mut u32) {
    // Advance base by a day when the timestamp wraps
    // Store the last timestamp to keep track, assuming it goes forward
    // Ignore if it goes slightly backward, should not happen but could
    // between different records
    if *last < *ts {
        *last = *ts;
    } else if (*last - *ts) > 1000 {
        *offset += 24 * 60 * 60;
        *last = *ts;
    }

    *ts += *offset;
}

impl Record<'_> {
    pub fn fix_timestamp(mut self, offset: &mut u32, last: &mut u32) -> Self {
        match self {
            Record::B(ref mut rec) => ts_offset(&mut rec.fix.timestamp, offset, last),
            Record::E(ref mut t) | Record::F(ref mut t) | Record::K(ref mut t) => {
                ts_offset(&mut t.timestamp, offset, last)
            }
            _ => {}
        }
        self
    }

    pub fn has_timestamp(&self) -> bool {
        matches!(
            self,
            Record::B(_) | Record::E(_) | Record::F(_) | Record::K(_)
        )
    }
}
