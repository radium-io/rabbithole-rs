use crate::model::document::{Document, DocumentItem, Included, PrimaryDataItem};
use crate::model::link::Links;
use crate::model::relationship::Relationships;
use crate::model::resource::{Attributes, Resource};
use crate::model::Meta;
use serde::{Deserialize, Serialize};

pub trait Entity: Serialize
where
    for<'de> Self: Deserialize<'de>,
{
    #[doc(hidden)]
    fn ty(&self) -> String;
    #[doc(hidden)]
    fn id(&self) -> String;
    #[doc(hidden)]
    fn attributes(&self) -> Attributes;
    #[doc(hidden)]
    fn links(&self) -> Links;
    #[doc(hidden)]
    fn relationships(&self) -> Relationships;
    #[doc(hidden)]
    fn included(&self) -> Included;
    #[doc(hidden)]
    fn meta(&self) -> Meta;
}

impl<T> From<&T> for Resource
where
    T: Entity,
{
    fn from(entity: &T) -> Self {
        Self {
            ty: entity.ty(),
            id: entity.id(),
            attributes: entity.attributes(),
            relationships: entity.relationships(),
            links: entity.links(),
            meta: entity.meta(),
        }
    }
}

impl<T> From<&T> for DocumentItem
where
    T: Entity,
{
    fn from(entity: &T) -> Self {
        let res: Resource = entity.into();
        DocumentItem::PrimaryData(Some((PrimaryDataItem::Single(Box::new(res)), entity.included())))
    }
}

impl<T> From<&T> for Document
where
    T: Entity,
{
    fn from(entity: &T) -> Self {
        let item: DocumentItem = entity.into();
        Self { item, links: None, meta: None, jsonapi: None }
    }
}
