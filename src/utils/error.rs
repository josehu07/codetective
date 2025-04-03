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
    Random(String),
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

    pub(crate) fn random(msg: impl ToString) -> Self {
        ApiKeyCheckError::Random(msg.to_string())
    }
}

impl fmt::Display for ApiKeyCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiKeyCheckError::Parse(msg) => write!(f, "Parse error: {}", msg),
            ApiKeyCheckError::Status(msg) => write!(f, "Status error: {}", msg),
            ApiKeyCheckError::Limit(msg) => write!(f, "Limit error: {}", msg),
            ApiKeyCheckError::Ascii(msg) => write!(f, "Ascii error: {}", msg),
            ApiKeyCheckError::Random(msg) => write!(f, "Random error: {}", msg),
        }
    }
}

impl Error for ApiKeyCheckError {}

impl From<reqwest::header::InvalidHeaderValue> for ApiKeyCheckError {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Self {
        ApiKeyCheckError::parse(err)
    }
}

impl From<reqwest::Error> for ApiKeyCheckError {
    fn from(err: reqwest::Error) -> Self {
        ApiKeyCheckError::status(err)
    }
}

impl From<getrandom::Error> for ApiKeyCheckError {
    fn from(err: getrandom::Error) -> Self {
        ApiKeyCheckError::random(err)
    }
}

/// Error type for code import validation.
#[derive(Clone, PartialEq, Debug)]
pub(crate) enum CodeImportError {
    Parse(String),
    Exists(String),
    Exten(String),
    Status(String),
    Limit(String),
    Ascii(String),
    GitHub(String),
    Upload(String),
}

impl CodeImportError {
    pub(crate) fn parse(msg: impl ToString) -> Self {
        CodeImportError::Parse(msg.to_string())
    }

    pub(crate) fn exists(msg: impl ToString) -> Self {
        CodeImportError::Exists(msg.to_string())
    }

    pub(crate) fn exten(msg: impl ToString) -> Self {
        CodeImportError::Exten(msg.to_string())
    }

    pub(crate) fn status(msg: impl ToString) -> Self {
        CodeImportError::Status(msg.to_string())
    }

    pub(crate) fn limit(msg: impl ToString) -> Self {
        CodeImportError::Limit(msg.to_string())
    }

    pub(crate) fn ascii(msg: impl ToString) -> Self {
        CodeImportError::Ascii(msg.to_string())
    }

    pub(crate) fn github(msg: impl ToString) -> Self {
        CodeImportError::GitHub(msg.to_string())
    }

    pub(crate) fn upload(msg: impl ToString) -> Self {
        CodeImportError::Upload(msg.to_string())
    }
}

impl fmt::Display for CodeImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeImportError::Parse(msg) => write!(f, "Parse error: {}", msg),
            CodeImportError::Exists(msg) => write!(f, "Exists error: {}", msg),
            CodeImportError::Exten(msg) => write!(f, "Extension error: {}", msg),
            CodeImportError::Status(msg) => write!(f, "Status error: {}", msg),
            CodeImportError::Limit(msg) => write!(f, "Limit error: {}", msg),
            CodeImportError::Ascii(msg) => write!(f, "Ascii error: {}", msg),
            CodeImportError::GitHub(msg) => write!(f, "GitHub error: {}", msg),
            CodeImportError::Upload(msg) => write!(f, "Upload error: {}", msg),
        }
    }
}

impl Error for CodeImportError {}

impl From<url::ParseError> for CodeImportError {
    fn from(err: url::ParseError) -> Self {
        CodeImportError::parse(err)
    }
}

impl From<gloo_file::FileReadError> for CodeImportError {
    fn from(err: gloo_file::FileReadError) -> Self {
        CodeImportError::parse(err)
    }
}

impl From<reqwest::header::InvalidHeaderValue> for CodeImportError {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Self {
        CodeImportError::parse(err)
    }
}

impl From<std::io::Error> for CodeImportError {
    fn from(err: std::io::Error) -> Self {
        CodeImportError::upload(err)
    }
}

impl From<reqwest::Error> for CodeImportError {
    fn from(err: reqwest::Error) -> Self {
        CodeImportError::status(err)
    }
}

/// Error type for detection API call.
#[derive(Clone, PartialEq, Debug)]
pub(crate) enum ApiMakeCallError {
    Parse(String),
    Status(String),
}

impl ApiMakeCallError {
    pub(crate) fn parse(msg: impl ToString) -> Self {
        ApiMakeCallError::Parse(msg.to_string())
    }

    pub(crate) fn status(msg: impl ToString) -> Self {
        ApiMakeCallError::Status(msg.to_string())
    }
}

impl fmt::Display for ApiMakeCallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiMakeCallError::Parse(msg) => write!(f, "Parse error: {}", msg),
            ApiMakeCallError::Status(msg) => write!(f, "Status error: {}", msg),
        }
    }
}

impl Error for ApiMakeCallError {}

impl From<serde_json::Error> for ApiMakeCallError {
    fn from(err: serde_json::Error) -> Self {
        ApiMakeCallError::parse(err)
    }
}

impl From<reqwest::header::InvalidHeaderValue> for ApiMakeCallError {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Self {
        ApiMakeCallError::parse(err)
    }
}

impl From<reqwest::Error> for ApiMakeCallError {
    fn from(err: reqwest::Error) -> Self {
        ApiMakeCallError::status(err)
    }
}
