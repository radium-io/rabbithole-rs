use http::uri::InvalidUri;
use rsql_rs::error::ParserError;
use std::num::ParseIntError;
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

impl From<std::num::ParseIntError> for RabbitholeError {
    fn from(err: ParseIntError) -> Self { RabbitholeError::Unhandled(Box::new(err)) }
}

impl From<rsql_rs::error::ParserError> for RabbitholeError {
    fn from(err: ParserError) -> Self { RabbitholeError::Unhandled(Box::new(err)) }
}
