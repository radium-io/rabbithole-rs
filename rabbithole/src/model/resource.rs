use crate::model::link::Links;
use crate::model::relationship::{Relationships};
use crate::model::{Id, Meta};
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::FromIterator;

pub type ResourceIdentifiers = Vec<ResourceIdentifier>;
pub type Resources = Vec<Resource>;

lazy_static! {
    static ref INVALID_ATTR_FIELDS: HashSet<&'static str> =
        HashSet::from_iter(vec!["relationships", "links", "type", "id"]);
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Attributes(HashMap<String, Value>);

impl From<HashMap<String, Value>> for Attributes {
    fn from(mut map: HashMap<String, Value>) -> Self {
        for &f in &INVALID_ATTR_FIELDS as &HashSet<&str> {
            map.remove(f);
        }
        Self(map)
    }
}

impl Attributes {
    fn is_empty(&self) -> bool { self.0.is_empty() }

    fn insert(&mut self, key: impl ToString, value: Value) -> Option<Value> {
        let key = key.to_string();
        if INVALID_ATTR_FIELDS.contains(&key.as_str()) {
            None
        } else {
            self.0.insert(key, value)
        }
    }

    fn get(&self, key: impl ToString) -> Option<&Value> { self.0.get(&key.to_string()) }
}

/// Valid Resource Identifier (can be None)
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum IdentifierData {
    Single(Option<ResourceIdentifier>),
    Multiple(ResourceIdentifiers),
}

impl Default for IdentifierData {
    fn default() -> Self { IdentifierData::Single(None) }
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
    #[serde(skip_serializing_if = "Attributes::is_empty")]
    #[serde(default)]
    pub attributes: Attributes,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub relationships: Relationships,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub links: Links,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub meta: Meta,
}
