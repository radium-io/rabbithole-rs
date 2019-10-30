use crate::model::link::Links;
use crate::model::relationship::Relationships;
use crate::model::{Id, Meta};
use serde_json::Value;
use std::collections::HashMap;

pub type ResourceIdentifiers = Vec<ResourceIdentifier>;
pub type Resources = Vec<Resource>;
pub type ResourceAttributes = HashMap<String, Value>;

/// Valid Resource Identifier (can be None)
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum IdentifierData {
    Single(ResourceIdentifier),
    Multiple(ResourceIdentifiers),
}

/// Resource Identifier
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ResourceIdentifier {
    #[serde(rename = "type")]
    pub ty: String,
    pub id: Id,
}

/// JSON-API Resource
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Resource {
    #[serde(rename = "type")]
    pub ty: String,
    pub id: Id,
    #[serde(default)]
    pub attributes: ResourceAttributes,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationships: Option<Relationships>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
}
