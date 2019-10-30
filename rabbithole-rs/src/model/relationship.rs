use crate::model::link::Links;
use crate::model::resource::IdentifierData;
use crate::model::Meta;
use std::collections::HashMap;

pub type Relationships = HashMap<String, Relationship>;

/// Relationship with another object
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Relationship {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<IdentifierData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
}
