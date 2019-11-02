use crate::model::document::{Document, DocumentItem, Included, PrimaryDataItem};
use crate::model::link::{Link, Links};
use crate::model::relationship::{RelationshipLinks, Relationships};
use crate::model::resource::{Attributes, Resource, ResourceIdentifier};
use crate::RbhOptionRes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::FromIterator;

pub trait Entity: Serialize {
    #[doc(hidden)]
    fn ty(&self) -> Option<String>;
    #[doc(hidden)]
    fn id(&self) -> Option<String>;
    #[doc(hidden)]
    fn attributes(&self) -> Option<Attributes>;
    #[doc(hidden)]
    fn links(&self, uri: &str) -> RbhOptionRes<Links> {
        if let (Some(ty), Some(id)) = (self.ty(), self.id()) {
            let slf = format!("{uri}/{ty}/{id}", uri = uri, ty = ty, id = id).parse::<Link>()?;
            Ok(Some(HashMap::from_iter(vec![("self".into(), slf)])))
        } else {
            Ok(None)
        }
    }
    #[doc(hidden)]
    fn relationships(&self, uri: &str) -> RbhOptionRes<Relationships>;
    #[doc(hidden)]
    fn included(&self, uri: &str) -> RbhOptionRes<Included>;

    fn to_document(&self, uri: &str) -> RbhOptionRes<Document> {
        if let Some(item) = self.to_document_item(uri)? {
            Ok(Some(Document { item, ..Default::default() }))
        } else {
            Ok(None)
        }
    }

    fn to_document_item(&self, uri: &str) -> RbhOptionRes<DocumentItem> {
        if let (Some(res), Some(included)) = (self.to_resource(uri)?, self.included(uri)?) {
            let primary = (PrimaryDataItem::Single(Box::new(res)), included);
            Ok(Some(DocumentItem::PrimaryData(Some(primary))))
        } else {
            Ok(None)
        }
    }

    fn to_relationship_links(
        &self, field_name: &str, uri: &str,
    ) -> RbhOptionRes<RelationshipLinks> {
        if let (Some(ty), Some(id)) = (self.ty(), self.id()) {
            let slf = format!(
                "{uri}/{ty}/{id}/relationships/{field_name}",
                uri = uri,
                ty = ty,
                id = id,
                field_name = field_name
            );
            let slf = slf.parse::<Link>()?;
            let related = format!(
                "{uri}/{ty}/{id}/{field_name}",
                uri = uri,
                ty = ty,
                id = id,
                field_name = field_name
            );
            let related = related.parse::<Link>()?;
            let links: RelationshipLinks =
                HashMap::from_iter(vec![("self".into(), slf), ("related".into(), related)]).into();
            Ok(Some(links))
        } else {
            Ok(None)
        }
    }

    fn to_resource_identifier(&self) -> Option<ResourceIdentifier> {
        if let (Some(ty), Some(id)) = (self.ty(), self.id()) {
            Some(ResourceIdentifier { ty, id })
        } else {
            None
        }
    }

    fn to_resource(&self, uri: &str) -> RbhOptionRes<Resource> {
        if let (Some(ty), Some(id), Some(attributes), Some(relationships), Some(links)) =
            (self.ty(), self.id(), self.attributes(), self.relationships(uri)?, self.links(uri)?)
        {
            Ok(Some(Resource { ty, id, attributes, relationships, links, ..Default::default() }))
        } else {
            Ok(None)
        }
    }
}

impl<T: Entity> Entity for Option<T> {
    fn ty(&self) -> Option<String> { self.as_ref().and_then(|s| s.ty()) }

    fn id(&self) -> Option<String> { self.as_ref().and_then(|s| s.id()) }

    fn attributes(&self) -> Option<Attributes> { self.as_ref().and_then(|s| s.attributes()) }

    fn relationships(&self, uri: &str) -> RbhOptionRes<Relationships> {
        if let Some(s) = self.as_ref() {
            s.relationships(uri)
        } else {
            Ok(None)
        }
    }

    fn included(&self, uri: &str) -> RbhOptionRes<Included> {
        if let Some(s) = self.as_ref() {
            s.included(uri)
        } else {
            Ok(None)
        }
    }
}

impl<T: Entity> Entity for Box<T> {
    fn ty(&self) -> Option<String> { self.as_ref().ty() }

    fn id(&self) -> Option<String> { self.as_ref().id() }

    fn attributes(&self) -> Option<Attributes> { self.as_ref().attributes() }

    fn relationships(&self, uri: &str) -> RbhOptionRes<Relationships> {
        self.as_ref().relationships(uri)
    }

    fn included(&self, uri: &str) -> RbhOptionRes<Included> { self.as_ref().included(uri) }
}

impl<'de, T: Entity> Entity for &T
where
    T: Deserialize<'de>,
{
    fn ty(&self) -> Option<String> { (*self).ty() }

    fn id(&self) -> Option<String> { (*self).id() }

    fn attributes(&self) -> Option<Attributes> { (*self).attributes() }

    fn relationships(&self, uri: &str) -> RbhOptionRes<Relationships> { (*self).relationships(uri) }

    fn included(&self, uri: &str) -> RbhOptionRes<Included> { (*self).included(uri) }
}
