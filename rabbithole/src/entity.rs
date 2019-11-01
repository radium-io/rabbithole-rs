use crate::model::document::{Document, DocumentItem, Included, PrimaryDataItem};
use crate::model::link::{Link, Links};
use crate::model::relationship::{RelationshipLinks, Relationships};
use crate::model::resource::{Attributes, Resource, ResourceIdentifier};
use crate::RbhResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::FromIterator;

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
    fn links(&self, uri: &str) -> RbhResult<Links> {
        let slf = format!("{uri}/{ty}/{id}", uri = uri, ty = self.ty(), id = self.id())
            .parse::<Link>()?;
        Ok(HashMap::from_iter(vec![("self".into(), slf)]).into())
    }
    #[doc(hidden)]
    fn relationships(&self, uri: &str) -> RbhResult<Relationships>;
    #[doc(hidden)]
    fn included(&self, uri: &str) -> RbhResult<Included>;

    fn to_document(&self, uri: &str) -> RbhResult<Document> {
        Ok(Document { item: self.to_document_item(uri)?, ..Default::default() })
    }

    fn to_document_item(&self, uri: &str) -> RbhResult<DocumentItem> {
        Ok(DocumentItem::PrimaryData(Some((
            PrimaryDataItem::Single(Box::new(self.to_resource(uri)?)),
            self.included(uri)?,
        ))))
    }

    fn to_resource(&self, uri: &str) -> RbhResult<Resource> {
        Ok(Resource {
            ty: self.ty(),
            id: self.id(),
            attributes: self.attributes(),
            relationships: self.relationships(uri)?,
            links: self.links(uri)?,
            ..Default::default()
        })
    }

    fn to_relationship_links(&self, field_name: &str, uri: &str) -> RbhResult<RelationshipLinks> {
        let slf = format!(
            "{uri}/{ty}/{id}/relationships/{field_name}",
            uri = uri,
            ty = self.ty(),
            id = self.id(),
            field_name = field_name
        );
        let slf = slf.parse::<Link>()?;
        let related = format!(
            "{uri}/{ty}/{id}/{field_name}",
            uri = uri,
            ty = self.ty(),
            id = self.id(),
            field_name = field_name
        );
        let related = related.parse::<Link>()?;
        let links: Links =
            HashMap::from_iter(vec![("self".into(), slf), ("related".into(), related)]);
        Ok(links.into())
    }

    fn to_resource_identifier(&self) -> ResourceIdentifier {
        ResourceIdentifier { ty: self.ty(), id: self.id() }
    }
}
