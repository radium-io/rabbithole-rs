use crate::model::Meta;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

pub type Links = HashMap<String, Link>;

#[derive(Debug,Serialize,Deserialize,Eq,PartialEq,Clone)]
pub struct WrappedUri(#[serde(with = "http_serde::uri")] http::Uri);

impl FromStr for Link {
    type Err = http::uri::InvalidUri;

    fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(Link::Raw(s.parse()?)) }
}

impl From<http::Uri> for Link {
    fn from(r: http::Uri) -> Self { Link::Raw(r) }
}

impl From<Link> for http::Uri {
    fn from(link: Link) -> Self {
        match link {
            Link::Raw(raw) => raw,
            Link::Object { href, .. } => href,
        }
    }
}

impl From<&Link> for http::Uri {
    fn from(link: &Link) -> Self {
        match link {
            Link::Raw(raw) => raw.to_owned(),
            Link::Object { href, .. } => href.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Link {
    #[serde(with = "http_serde::uri")]
    Raw(http::Uri),
    Object { 
        #[serde(with = "http_serde::uri")]
        href: http::Uri, 
        meta: Meta 
    },
}

impl Link {
    pub fn new(uri: &str, path: http::Uri) -> Link { 
        let base = uri.parse::<url::Url>().unwrap().join(path.to_string().as_str()).unwrap();
        base.to_string().parse::<http::Uri>().unwrap().into()
     }

    pub fn slf(uri: &str, path: http::Uri) -> (String, Link) { ("self".into(), Link::new(uri, path)) }
}