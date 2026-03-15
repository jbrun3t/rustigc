use winnow::error::{ContextError, ParseError};

#[derive(thiserror::Error, Debug)]
pub enum LError {
    #[error("Parse error: {0}")]
    Parse(String),

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

pub type LResult<T> = std::result::Result<T, LError>;
