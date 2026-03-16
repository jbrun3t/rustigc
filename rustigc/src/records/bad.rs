//! Bad record
//!
//! Catch all parser for bad records

use winnow::error::Result as PResult;
use winnow::token::take_while;
use winnow::prelude::*;

use super::Record;

/// Match bad record and store them for analysis in the rawlog
pub fn bad_record<'a>(input: &mut &'a [u8]) -> PResult<Record<'a>> {
    // Not using the classic (till_line_ending, line_ending) because it
    // fails with \r\r\n
    // ... got to be sure the carriage returned I guess :(
    (
        take_while(0.., |c| c != b'\n'),
        "\n".as_bytes()
    )
        .with_taken()
        .map(|(_, taken): (_, &[u8])| {
            eprintln!(
                "Bad Record: {}",
                std::str::from_utf8(taken).unwrap_or_else(|_| "Non ASCII")
            );
            Record::BAD(taken)
        }).parse_next(input)
}
