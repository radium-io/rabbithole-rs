use thiserror::Error;

#[derive(Error, Debug)]
pub enum RabbitholeError {
    #[error("Unhandled")]
    Unhandled(#[source] Box<dyn std::error::Error>),
}
