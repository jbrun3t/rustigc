// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! IGC record parsers, one module per record type.

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
/// One line of an IGC file, named after the letter it starts with.
///
/// `Display` prints it back as IGC; `{:#}` prints it for a human instead.
pub enum Record<'a> {
    /// Flight recorder identification.
    A(Recorder),
    /// Position fix.
    B(RawFix<'a>),
    /// Declared task.
    C(Task),
    /// GPS datum and differential GPS station.
    D(TextEvent<'a>),
    /// Pilot-initiated event.
    E(TimedEvent<'a>),
    /// Satellite constellation used for the fixes that follow.
    F(TimedEvent<'a>),
    /// Security signature.
    G(TextEvent<'a>),
    /// Flight metadata.
    H(Header),
    /// Extension fields appended to every [`Record::B`].
    I(Extensions<'a>),
    /// Extension fields appended to every [`Record::K`].
    J(Extensions<'a>),
    /// Periodic sensor data, laid out by the [`Record::J`] definitions.
    K(TimedEvent<'a>),
    /// Free-form comment.
    L(TextEvent<'a>),
    /// A line no parser recognized, kept verbatim.
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
    } else if (*last - *ts) > 1_000_000 {
        *offset += 24 * 60 * 60 * 1000;
        *last = *ts;
    }

    *ts += *offset;
}

impl Record<'_> {
    /// Carries a timestamp past midnight, advancing `offset` by a day whenever it wraps.
    ///
    /// `last` is the previous timestamp seen, which is how the wrap is spotted.
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

    /// Whether this record carries a time of its own.
    pub fn has_timestamp(&self) -> bool {
        matches!(
            self,
            Record::B(_) | Record::E(_) | Record::F(_) | Record::K(_)
        )
    }
}
