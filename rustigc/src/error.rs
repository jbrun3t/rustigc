// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

use winnow::error::{ContextError, ParseError};

/// What went wrong turning bytes into a [`Log`].
///
/// [`Log`]: crate::Log
#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DecodeError {
    /// No record could be read, which only happens on empty input: any other line decodes, as
    /// [`Record::Bad`] if nothing else.
    ///
    /// [`Record::Bad`]: crate::Record::Bad
    #[error("no record could be read at byte {offset}")]
    Parse { offset: usize },

    /// A [`RawLog`] with no records.
    ///
    /// [`RawLog`]: crate::RawLog
    #[error("no records to decode")]
    Empty,

    /// At least 80% of the records are invalid.
    #[error("too many invalid records")]
    TooManyBadRecords,
}

impl From<ParseError<&[u8], ContextError>> for DecodeError {
    fn from(err: ParseError<&[u8], ContextError>) -> Self {
        Self::Parse {
            offset: err.offset(),
        }
    }
}

/// What went wrong against a track.
#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TrackError {
    /// A fix index past the end of the track.
    #[error("fix {index} is out of range, the track holds {len}")]
    FixOutOfRange { index: usize, len: usize },

    /// A window that selects nothing: `start` at or past `stop`.
    #[error("window {start}..{stop} is empty or inverted")]
    InvalidWindow { start: usize, stop: usize },

    /// Fewer than the two points the shortest task needs.
    #[error("{points} point cannot be scored, 2 are the minimum")]
    TooFewPoints { points: usize },

    /// A point that is not a finite latitude and longitude in decimal degrees.
    #[error("point {index} is not a finite coordinate in degrees")]
    BadCoordinate { index: usize },
}

/// What went wrong scoring.
#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScoreError {
    /// Not one of the preset ids.
    #[error("unknown preset")]
    UnknownPreset,

    /// The track or window cannot be scored.
    #[error(transparent)]
    Track(#[from] TrackError),
}
