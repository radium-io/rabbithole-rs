use crate::model::Meta;
use core::fmt;
use serde::de::Visitor;

use serde::export::Formatter;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::str::FromStr;

pub type Links = HashMap<String, Link>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RawUri(http::Uri);

impl FromStr for RawUri {
    type Err = http::uri::InvalidUri;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(RawUri(s.parse()?))
    }
}

impl FromStr for Link {
    type Err = http::uri::InvalidUri;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Link::Raw(s.parse()?))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Link {
    Raw(RawUri),
    Object { href: RawUri, meta: Meta },
}

impl Serialize for RawUri {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("RawUri", &self.0.to_string())
    }
}

struct RawUriVisitor;
impl<'de> Visitor<'de> for RawUriVisitor {
    type Value = RawUri;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("URI valid String")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Ok(uri) = v.parse::<http::Uri>() {
            Ok(RawUri(uri))
        } else {
            Err(serde::de::Error::custom("The string is not an invalid URI"))
        }
    }
}

impl<'de> Deserialize<'de> for RawUri {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(RawUriVisitor)
    }
}
