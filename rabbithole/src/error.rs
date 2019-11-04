use http::uri::InvalidUri;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RabbitholeError {
    #[error("Invalid URI")]
    InvalidUri(#[source] http::uri::InvalidUri),
    #[error(
        "Invalid pagination type: {0}, the valid types: `OffsetBased`, `PageBased`, `CursorBased`"
    )]
    InvalidPageType(String),
    #[error("Invalid filter type: {0}, the valid types: `Rsql`")]
    InvalidFilterType(String),
    #[error("Unhandled")]
    Unhandled(#[source] Box<dyn std::error::Error>),
}

impl From<http::uri::InvalidUri> for RabbitholeError {
    fn from(err: InvalidUri) -> Self { RabbitholeError::InvalidUri(err) }
}

from_external_error!(std::num::ParseIntError, rsql_rs::error::ParserError, std::str::Utf8Error);
