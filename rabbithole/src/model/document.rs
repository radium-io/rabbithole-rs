use crate::model::error::Errors;
use crate::model::link::Links;

use crate::model::resource::{Resource, ResourceIdentifier, Resources};
use crate::model::{JsonApiInfo, Meta};
use core::fmt;
use serde::de::{MapAccess, Visitor};

use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;

pub type Included = HashMap<ResourceIdentifier, Resource>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum PrimaryDataItem {
    Single(Box<Resource>),
    Multiple(Resources),
}

impl PrimaryDataItem {
    pub fn data(&self) -> Vec<Resource> {
        match self {
            PrimaryDataItem::Single(res) => vec![res.as_ref().clone()],
            PrimaryDataItem::Multiple(vec) => vec.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentItem {
    PrimaryData(Option<(PrimaryDataItem, Included)>),
    Errors(Errors),
}

impl Default for DocumentItem {
    fn default() -> Self { DocumentItem::PrimaryData(None) }
}

/// The specification refers to this as a top-level `document`
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub item: DocumentItem,
    pub links: Links,
    pub meta: Meta,
    pub jsonapi: Option<JsonApiInfo>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            item: Default::default(),
            links: Default::default(),
            meta: Default::default(),
            jsonapi: Default::default(),
        }
    }
}

impl Document {
    pub fn null(links: Links, meta: Meta) -> Self {
        Self {
            links,
            meta,
            ..Default::default()
        }
    }

    pub fn into_single(self) -> Result<(Box<Resource>, Included), Self> {
        if let DocumentItem::PrimaryData(Some((PrimaryDataItem::Single(resource), included))) =
            self.item
        {
            Ok((resource, included))
        } else {
            Err(self)
        }
    }

    pub fn into_multiple(self) -> Result<(Vec<Resource>, Included), Self> {
        if let DocumentItem::PrimaryData(Some((PrimaryDataItem::Multiple(resources), included))) =
            self.item
        {
            Ok((resources, included))
        } else {
            Err(self)
        }
    }

    pub fn single_resource(resource: Resource, included: Included) -> Self {
        Self {
            item: DocumentItem::PrimaryData(Some((
                PrimaryDataItem::Single(Box::new(resource)),
                included,
            ))),
            ..Default::default()
        }
    }

    pub fn multiple_resources(resources: Vec<Resource>, included: Included) -> Self {
        Self {
            item: DocumentItem::PrimaryData(Some((PrimaryDataItem::Multiple(resources), included))),
            ..Default::default()
        }
    }

    pub fn extend_meta(&mut self, meta: Meta) { self.meta.extend(meta.into_iter()); }

    pub fn extend_links(&mut self, links: Links) { self.links.extend(links.into_iter()); }
}

impl Serialize for Document {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Document", 4)?;
        match self.item {
            DocumentItem::PrimaryData(Some((ref data, ref included))) => {
                state.serialize_field("data", data)?;
                if !included.is_empty() {
                    state.serialize_field(
                        "included",
                        &included.values().collect::<Vec<&Resource>>(),
                    )?;
                }
            },
            DocumentItem::Errors(ref errors) => {
                state.serialize_field("errors", errors)?;
            },
            _ => state.serialize_field("data", &serde_json::Value::Null)?,
        }

        if !self.links.is_empty() {
            state.serialize_field("links", &self.links)?;
        }
        if !self.meta.is_empty() {
            state.serialize_field("meta", &self.meta)?;
        }
        if let Some(ref jsonapi) = self.jsonapi {
            state.serialize_field("jsonapi", jsonapi)?;
        }

        state.end()
    }
}

struct DocumentVisitor;

impl<'de> Visitor<'de> for DocumentVisitor {
    type Value = Document;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a JSON Object")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(serde_json::from_str::<Document>(v).unwrap())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut links = None;
        let mut meta = None;
        let mut jsonapi = None;
        let mut data = None;
        let mut included = None;
        let mut errors = None;

        while let Some((key, value)) = map.next_entry::<String, Value>()? {
            match key.as_str() {
                "links" if links.is_none() => match serde_json::from_value::<Links>(value) {
                    Ok(new_data) => links = Some(new_data),
                    Err(err) => return Err(serde::de::Error::custom(err)),
                },
                "links" => return Err(serde::de::Error::duplicate_field("links")),
                "meta" if meta.is_none() => match serde_json::from_value::<Meta>(value) {
                    Ok(new_data) => meta = Some(new_data),
                    Err(err) => return Err(serde::de::Error::custom(err)),
                },
                "meta" => return Err(serde::de::Error::duplicate_field("meta")),
                "jsonapi" if jsonapi.is_none() => {
                    match serde_json::from_value::<JsonApiInfo>(value) {
                        Ok(new_data) => jsonapi = Some(new_data),
                        Err(err) => return Err(serde::de::Error::custom(err)),
                    }
                },
                "jsonapi" => return Err(serde::de::Error::duplicate_field("jsonapi")),
                "data" if data.is_none() => {
                    match serde_json::from_value::<Option<PrimaryDataItem>>(value) {
                        Ok(new_data) => data = new_data,
                        Err(err) => return Err(serde::de::Error::custom(err)),
                    }
                },
                "data" => return Err(serde::de::Error::duplicate_field("data")),
                "included" if included.is_none() => {
                    match serde_json::from_value::<Vec<Resource>>(value) {
                        Ok(new_data) => {
                            let new_data: Included =
                                new_data.into_iter().map(|r| (r.id.clone(), r)).collect();
                            included = Some(new_data);
                        },
                        Err(err) => return Err(serde::de::Error::custom(err)),
                    }
                },
                "included" => return Err(serde::de::Error::duplicate_field("included")),
                "errors" if errors.is_none() => match serde_json::from_value::<Errors>(value) {
                    Ok(new_data) => errors = Some(new_data),
                    Err(err) => return Err(serde::de::Error::custom(err)),
                },
                "errors" => return Err(serde::de::Error::duplicate_field("errors")),
                _ => {},
            }
        }

        let item = match (data, included, errors) {
            (Some(data), Some(included), None) => DocumentItem::PrimaryData(Some((data, included))),
            (Some(data), None, None) => DocumentItem::PrimaryData(Some((data, Default::default()))),
            (None, None, Some(errors)) => DocumentItem::Errors(errors),
            (None, Some(_), _) => {
                return Err(serde::de::Error::custom(
                    "field `included` cannot exist without `data`",
                ));
            },
            (None, None, None) => DocumentItem::PrimaryData(None),
            _ => {
                return Err(serde::de::Error::custom(
                    "field `data` and `errors` cannot exists in the same document",
                ));
            },
        };

        Ok(Document {
            item,
            links: links.unwrap_or_default(),
            meta: meta.unwrap_or_default(),
            jsonapi,
        })
    }
}

impl<'de> Deserialize<'de> for Document {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(DocumentVisitor)
    }
}
