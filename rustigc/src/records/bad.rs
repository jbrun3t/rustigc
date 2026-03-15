//! Bad record
//!
//! Catch all parser for bad records

use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::terminated;
use winnow::error::Result as PResult;
use winnow::prelude::*;

use super::Record;

pub fn bad_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    terminated(till_line_ending, line_ending)
        .map(|rec: &[u8]| {
            eprintln!(
                "Bad Record: {}",
                std::str::from_utf8(rec).unwrap_or_else(|_| "Non ASCII")
            );
            Record::BAD(rec)
        })
        .parse_next(input)
}
