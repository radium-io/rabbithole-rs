pub mod document;
pub mod error;
pub mod link;
pub mod pagination;
pub mod patch;
pub mod relationship;
pub mod resource;

use serde_json;
use serde_json::Value;
use std::collections::HashMap;

/// Meta-data object, can contain any data
pub type Meta = HashMap<String, Value>;
pub type Id = String;

/// Optional `JsonApiDocument` payload identifying the JSON-API version the server implements
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct JsonApiInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
}
