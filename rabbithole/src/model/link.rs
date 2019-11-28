use crate::model::Meta;
use core::fmt;
use serde::de::Visitor;

use http::Uri;
use serde::export::Formatter;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

use std::str::FromStr;

pub type Links = HashMap<String, Link>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RawUri(pub(crate) http::Uri);

impl RawUri {
    fn append_to(self, base_url: &str) -> RawUri {
        let base = base_url.parse::<url::Url>().unwrap().join(&self.0.to_string()).unwrap();
        RawUri(base.to_string().parse::<http::Uri>().unwrap())
    }

    pub fn query(&self) -> Option<&str> { self.0.query() }

    pub fn path(&self) -> &str { self.0.path() }
}

impl FromStr for RawUri {
    type Err = http::uri::InvalidUri;

    fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(RawUri(s.parse()?)) }
}

impl FromStr for Link {
    type Err = http::uri::InvalidUri;

    fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(Link::Raw(s.parse()?)) }
}

impl From<RawUri> for Link {
    fn from(r: RawUri) -> Self { Link::Raw(r) }
}

impl From<http::Uri> for RawUri {
    fn from(uri: Uri) -> Self { RawUri(uri) }
}

impl From<&http::Uri> for RawUri {
    fn from(uri: &Uri) -> Self { RawUri(uri.clone()) }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Link {
    Raw(RawUri),
    Object { href: RawUri, meta: Meta },
}

impl Link {
    pub fn new(uri: &str, path: RawUri) -> Link { path.append_to(uri).into() }

    pub fn slf(uri: &str, path: RawUri) -> (String, Link) { ("self".into(), Link::new(uri, path)) }
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
