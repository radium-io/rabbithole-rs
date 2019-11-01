use crate::error::RabbitholeError;
use crate::model::document::{Document, DocumentItem, Included, PrimaryDataItem};
use crate::model::link::{Link, Links};
use crate::model::relationship::{Relationship, RelationshipLinks, Relationships};
use crate::model::resource::{Attributes, Resource, ResourceIdentifier};
use crate::RbhResult;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::iter::FromIterator;

pub trait Entity: Serialize
where
    for<'de> Self: Deserialize<'de>,
{
    #[doc(hidden)]
    fn ty(&self) -> Option<String>;
    #[doc(hidden)]
    fn id(&self) -> Option<String>;
    #[doc(hidden)]
    fn attributes(&self) -> Option<Attributes>;
    #[doc(hidden)]
    fn links(&self, uri: &str) -> RbhResult<Option<Links>> {
        let slf = format!("{uri}/{ty}/{id}", uri = uri, ty = self.ty(), id = self.id())
            .parse::<Link>()?;
        Ok(HashMap::from_iter(vec![("self".into(), slf)]))
    }
    #[doc(hidden)]
    fn relationships(&self, uri: &str) -> RbhResult<Relationships>;
    #[doc(hidden)]
    fn included(&self, uri: &str) -> RbhResult<Included>;

    fn to_document(&self, uri: &str) -> RbhResult<Document> {
        Ok(Document { item: self.to_document_item(uri)?, ..Default::default() })
    }

    fn to_document_item(&self, uri: &str) -> RbhResult<DocumentItem> {
        let primary =
            Some((PrimaryDataItem::Single(Box::new(self.to_resource(uri)?)), self.included(uri)?));
        Ok(DocumentItem::PrimaryData(primary))
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
}

impl<T: Entity> Entity for Option<T> {
    fn ty(&self) -> String { unimplemented!() }

    fn id(&self) -> String { unimplemented!() }

    fn attributes(&self) -> Attributes { unimplemented!() }

    fn relationships(
        &self, uri: &str,
    ) -> Result<HashMap<String, Relationship, RandomState>, RabbitholeError> {
        unimplemented!()
    }

    fn included(&self, uri: &str) -> Result<Vec<Resource>, RabbitholeError> { unimplemented!() }
}

pub trait EntityExt<T: Entity> {
    fn to_entity(&self) -> Option<&T>;
}

impl<T: Entity> EntityExt<T> for T {
    fn to_entity(&self) -> Option<&T> { Some(&self) }
}

impl<T: Entity> EntityExt<T> for Option<T> {
    fn to_entity(&self) -> Option<&T> { self.as_ref().and_then(|item| item.to_entity()) }
}
