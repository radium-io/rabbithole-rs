use crate::model::link::{Link, Links};
use crate::model::resource::IdentifierData;
use crate::model::Meta;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::FromIterator;

pub type Relationships = HashMap<String, Relationship>;
lazy_static! {
    static ref INVALID_RELAT_FIELDS: HashSet<&'static str> = HashSet::from_iter(vec!["type", "id"]);
}

/// Relationship with another object
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Relationship {
    pub data: IdentifierData,
    #[serde(skip_serializing_if = "RelationshipLinks::is_not_valid")]
    #[serde(default)]
    pub links: RelationshipLinks,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub meta: Meta,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct RelationshipLinks {
    #[serde(rename = "self")]
    #[serde(skip_serializing_if = "Option::is_none")]
    slf: Option<Link>,
    #[serde(skip_serializing_if = "Option::is_none")]
    related: Option<Link>,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    links: Links,
}

impl From<Links> for RelationshipLinks {
    fn from(mut links: Links) -> Self {
        let slf = links.remove("self");
        let related = links.remove("related");
        for &f in &INVALID_RELAT_FIELDS as &HashSet<&str> {
            links.remove(f);
        }
        Self { slf, related, links }
    }
}

impl RelationshipLinks {
    pub fn is_valid(&self) -> bool { !(self.slf.is_none() && self.related.is_none()) }

    fn is_not_valid(&self) -> bool { !self.is_valid() }

    pub fn get(&self, key: impl ToString) -> Option<&Link> {
        let key = key.to_string();
        if key == "self" {
            self.slf.as_ref()
        } else if key == "related" {
            self.related.as_ref()
        } else {
            self.links.get(&key)
        }
    }

    pub fn insert(&mut self, key: impl ToString, value: Link) -> Option<Link> {
        let key = key.to_string();
        if INVALID_RELAT_FIELDS.contains(&key.as_str()) {
            None
        } else if key == "self" {
            self.slf = Some(value);
            self.slf.clone()
        } else if key == "related" {
            self.related = Some(value);
            self.related.clone()
        } else {
            self.links.insert(key, value)
        }
    }
}
