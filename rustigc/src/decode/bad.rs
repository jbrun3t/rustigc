// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Bad record
//!
//! Catch all parser for bad records

use log::warn;
use winnow::combinator::{alt, eof, not, repeat};
use winnow::error::Result as PResult;
use winnow::prelude::*;
use winnow::token::{any, take_while};

use super::Record;

/// Match bad records and store them for analysis in the rawlog
pub fn bad_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    // Bad record condition:
    // 1st: handle bad records in the middle of the file, taking care
    //      to trash multiple carriage returns
    // 2nd: handle truncated last line
    alt((
        (take_while(0.., |c| c != b'\n'), "\n".as_bytes()).void(),
        (repeat(1.., (not(eof), any))).map(|_: ()| ()),
    ))
    .with_taken()
    .map(|(_, taken): (_, &[u8])| {
        warn!("Bad Record: {}", String::from_utf8_lossy(taken));
        Record::Bad(taken)
    })
    .parse_next(input)
}
