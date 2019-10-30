use crate::model::error::Errors;
use crate::model::link::Links;
use crate::model::resource::{Resource, Resources};
use crate::model::{JsonApiInfo, Meta};
use core::fmt;
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

pub type Included = Vec<Resource>;

/// Valid data Resource (can be None)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum PrimaryDataItem {
    Single(Box<Resource>),
    Multiple(Resources),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentItem {
    PrimaryData(Option<(PrimaryDataItem, Included)>),
    Errors(Errors),
}

impl Default for DocumentItem {
    fn default() -> Self {
        DocumentItem::PrimaryData(None)
    }
}

/// The specification refers to this as a top-level `document`
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document {
    pub item: DocumentItem,
    pub links: Option<Links>,
    pub meta: Option<Meta>,
    pub jsonapi: Option<JsonApiInfo>,
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
                    state.serialize_field("included", included)?;
                }
            }
            DocumentItem::Errors(ref errors) => {
                state.serialize_field("errors", errors)?;
            }
            _ => state.serialize_field("data", &serde_json::Value::Null)?,
        }

        if let Some(ref links) = self.links {
            state.serialize_field("links", links)?;
        }
        if let Some(ref meta) = self.meta {
            state.serialize_field("meta", meta)?;
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
        println!("visit_str: {}", v);
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
                }
                "jsonapi" => return Err(serde::de::Error::duplicate_field("jsonapi")),
                "data" if data.is_none() => {
                    match serde_json::from_value::<Option<PrimaryDataItem>>(value) {
                        Ok(new_data) => data = new_data,
                        Err(err) => return Err(serde::de::Error::custom(err)),
                    }
                }
                "data" => return Err(serde::de::Error::duplicate_field("data")),
                "included" if included.is_none() => {
                    match serde_json::from_value::<Included>(value) {
                        Ok(new_data) => included = Some(new_data),
                        Err(err) => return Err(serde::de::Error::custom(err)),
                    }
                }
                "included" => return Err(serde::de::Error::duplicate_field("included")),
                "errors" if errors.is_none() => match serde_json::from_value::<Errors>(value) {
                    Ok(new_data) => errors = Some(new_data),
                    Err(err) => return Err(serde::de::Error::custom(err)),
                },
                "errors" => return Err(serde::de::Error::duplicate_field("errors")),
                _ => {}
            }
        }

        let item = match (data, included, errors) {
            (Some(data), Some(included), None) => DocumentItem::PrimaryData(Some((data, included))),
            (Some(data), None, None) => DocumentItem::PrimaryData(Some((data, Default::default()))),
            (None, None, Some(errors)) => DocumentItem::Errors(errors),
            (None, Some(_), _) => {
                return Err(serde::de::Error::custom(
                    "field `included` cannot exist without `data`",
                ))
            }
            (None, None, None) => DocumentItem::PrimaryData(None),
            _ => {
                return Err(serde::de::Error::custom(
                    "field `data` and `errors` cannot exists in the same document",
                ));
            }
        };

        Ok(Document {
            item,
            links,
            meta,
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
