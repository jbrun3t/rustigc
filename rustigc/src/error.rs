// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

use winnow::error::{ContextError, ParseError};

/// What went wrong reading a log.
#[derive(thiserror::Error, Debug)]
pub enum LError {
    /// The bytes are not IGC: not one record could be read.
    #[error("Parse error: {0}")]
    Parse(String),

    /// IGC-shaped, but not usable — too much of it is invalid.
    #[error("D'oh: {0}")]
    Doh(String),
}

impl From<ParseError<&str, ContextError>> for LError {
    fn from(err: ParseError<&str, ContextError>) -> Self {
        LError::Parse(err.to_string())
    }
}

impl From<ParseError<&[u8], ContextError>> for LError {
    fn from(err: ParseError<&[u8], ContextError>) -> Self {
        LError::Parse(err.to_string())
    }
}

impl From<ContextError> for LError {
    fn from(err: ContextError) -> Self {
        LError::Parse(err.to_string())
    }
}

/// [`Result`] of this crate's parsing entry points.
pub type LResult<T> = std::result::Result<T, LError>;
