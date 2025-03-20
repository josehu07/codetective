//! Useful error types.

use std::error::Error;
use std::fmt;

/// Error type for API key validation.
#[derive(Clone, PartialEq, Debug)]
pub(crate) enum ApiKeyCheckError {
    Parse(String),
    Status(String),
    Limit(String),
    Ascii(String),
}

impl ApiKeyCheckError {
    pub(crate) fn parse(msg: impl ToString) -> Self {
        ApiKeyCheckError::Parse(msg.to_string())
    }

    pub(crate) fn status(msg: impl ToString) -> Self {
        ApiKeyCheckError::Status(msg.to_string())
    }

    pub(crate) fn limit(msg: impl ToString) -> Self {
        ApiKeyCheckError::Limit(msg.to_string())
    }

    pub(crate) fn ascii(msg: impl ToString) -> Self {
        ApiKeyCheckError::Ascii(msg.to_string())
    }
}

impl fmt::Display for ApiKeyCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiKeyCheckError::Parse(msg) => write!(f, "Parse error: {}", msg),
            ApiKeyCheckError::Status(msg) => write!(f, "Status error: {}", msg),
            ApiKeyCheckError::Limit(msg) => write!(f, "Limit error: {}", msg),
            ApiKeyCheckError::Ascii(msg) => write!(f, "Ascii error: {}", msg),
        }
    }
}

impl Error for ApiKeyCheckError {}

impl From<reqwest::Error> for ApiKeyCheckError {
    fn from(err: reqwest::Error) -> Self {
        ApiKeyCheckError::parse(err)
    }
}

impl From<reqwest::header::InvalidHeaderValue> for ApiKeyCheckError {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Self {
        ApiKeyCheckError::parse(err)
    }
}
