use crate::model::link::{Links, RawUri};
use crate::model::Meta;
use serde::{Deserialize, Serialize};

pub type Errors = Vec<Error>;

/// Error location
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ErrorSource {
    pub pointer: Option<RawUri>,
    pub parameter: Option<String>,
}

/// JSON-API Error
/// All fields are optional
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Error {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ErrorSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
}
