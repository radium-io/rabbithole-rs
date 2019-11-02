use http::uri::InvalidUri;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RabbitholeError {
    #[error("Invalid URI")]
    InvalidUri(#[source] http::uri::InvalidUri),
    #[error("Unhandled")]
    Unhandled(#[source] Box<dyn std::error::Error>),
}

impl From<http::uri::InvalidUri> for RabbitholeError {
    fn from(err: InvalidUri) -> Self { RabbitholeError::InvalidUri(err) }
}
