use winnow::error::{ContextError, ParseError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("D'oh: {0}")]
    Doh(String),
}

impl From<ParseError<&str, ContextError>> for Error {
    fn from(err: ParseError<&str, ContextError>) -> Self {
        Error::Parse(err.to_string())
    }
}

impl From<ParseError<&[u8], ContextError>> for Error {
    fn from(err: ParseError<&[u8], ContextError>) -> Self {
        Error::Parse(err.to_string())
    }
}

impl From<ContextError> for Error {
    fn from(err: ContextError) -> Self {
        Error::Parse(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
