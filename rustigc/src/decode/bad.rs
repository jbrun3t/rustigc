// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Catch-all parser for lines no other parser accepts.

use log::warn;
use winnow::combinator::{alt, eof, not, repeat};
use winnow::error::Result as PResult;
use winnow::prelude::*;
use winnow::token::{any, take_while};

use super::Record;

/// Consumes one line, or an unterminated last line, as [`Record::Bad`] and logs it.
pub fn bad_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
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
