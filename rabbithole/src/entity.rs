use crate::model::document::{Document, Included};
use crate::model::link::{Link, Links};
use crate::model::query::{FieldsQuery, IncludeQuery, Query};
use crate::model::relationship::{RelationshipLinks, Relationships};
use crate::model::resource::{Attributes, Resource, ResourceIdentifier};
use crate::RbhOptionRes;
use serde::Serialize;

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

    /// Returns the `included` field of this entity
    ///
    /// `include_query`: If exists, only the `included` item whose `ty` is in the `include_query`
    ///                  will be retained
    ///                  If exists but empty, means all `included` fields will be ignored
    ///                  If not exists, all the `included` fields will be retained   
    ///
    /// `fields_query`: For any resources whose `ty` is in the `fields_query`, their `relationship`
    ///                 and `attributes` will be filtered. Only the field name inside the `field_query`
    ///                 item will be retained
    #[doc(hidden)]
    fn included(
        &self, uri: &str, include_query: &Option<IncludeQuery>, fields_query: &Option<FieldsQuery>,
    ) -> RbhOptionRes<Included>;

    /// Returns a `Document` based on `query`. This function will do all of the actions databases should do in memory,
    /// using a trivial iter way. But I still recommend you guys implement `to_document` or `to_document_async` yourself
    /// for better performance
    fn to_document_automatically(&self, uri: &str, query: &Query) -> RbhOptionRes<Document> {
        if let (Some(res), Some(included)) = (
            self.to_resource(uri, &query.fields)?,
            self.included(uri, &query.include, &query.fields)?,
        ) {
            Ok(Some(Document::single_resource(res, included)))
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

    fn to_resource(&self, uri: &str, fields_query: &Option<FieldsQuery>) -> RbhOptionRes<Resource> {
        if let (Some(ty), Some(id), Some(mut attributes), Some(mut relationships), Some(links)) =
            (self.ty(), self.id(), self.attributes(), self.relationships(uri)?, self.links(uri)?)
        {
            if let Some(fields_query) = fields_query {
                for (k, vs) in fields_query.iter() {
                    if &ty == k {
                        attributes = attributes.retain(vs);
                        relationships.retain(|k, _| vs.contains(k));
                    }
                }
            }

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

    fn included(
        &self, uri: &str, include_query: &Option<IncludeQuery>, fields_query: &Option<FieldsQuery>,
    ) -> RbhOptionRes<Included> {
        if let Some(s) = self.as_ref() {
            s.included(uri, include_query, fields_query)
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

    fn included(
        &self, uri: &str, include_query: &Option<IncludeQuery>, fields_query: &Option<FieldsQuery>,
    ) -> RbhOptionRes<Included> {
        self.as_ref().included(uri, include_query, fields_query)
    }
}

impl<T: Entity> Entity for &T {
    fn ty(&self) -> Option<String> { (*self).ty() }

    fn id(&self) -> Option<String> { (*self).id() }

    fn attributes(&self) -> Option<Attributes> { (*self).attributes() }

    fn relationships(&self, uri: &str) -> RbhOptionRes<Relationships> { (*self).relationships(uri) }

    fn included(
        &self, uri: &str, include_query: &Option<IncludeQuery>, fields_query: &Option<FieldsQuery>,
    ) -> RbhOptionRes<Included> {
        (*self).included(uri, include_query, fields_query)
    }
}

#[cfg(feature = "unstable-vec-to-document")]
impl<T: Entity> Entity for [T] {
    fn ty(&self) -> Option<String> { None }

    fn id(&self) -> Option<String> { None }

    fn attributes(&self) -> Option<Attributes> { None }

    fn relationships(&self, _: &str) -> RbhOptionRes<Relationships> { Ok(None) }

    fn included(
        &self, uri: &str, include_query: &Option<IncludeQuery>, fields_query: &Option<FieldsQuery>,
    ) -> RbhOptionRes<Included> {
        let mut hashmap: HashMap<ResourceIdentifier, Resource> = Default::default();
        for e in self {
            if let Some(inc) = e.included(uri, include_query, fields_query)? {
                for i in inc {
                    hashmap.insert(ResourceIdentifier { ty: i.ty.clone(), id: i.id.clone() }, i);
                }
            }
        }

        Ok(Some(hashmap.values().cloned().collect()))
    }

    fn to_document_automatically(&self, uri: &str, query: &Query) -> RbhOptionRes<Document> {
        if let Some(included) = self.included(uri, &query.include, &query.fields)? {
            let mut reses = vec![];
            for e in self {
                if let Some(res) = e.to_resource(uri, &query.fields)? {
                    reses.push(res);
                }
            }
            Ok(Some(Document::multiple_resources(reses, included)))
        } else {
            Ok(None)
        }
    }
}

#[cfg(feature = "unstable-vec-to-document")]
impl<T: Entity> Entity for Vec<T> {
    fn ty(&self) -> Option<String> { None }

    fn id(&self) -> Option<String> { None }

    fn attributes(&self) -> Option<Attributes> { None }

    fn relationships(&self, _: &str) -> RbhOptionRes<Relationships> { Ok(None) }

    fn included(
        &self, uri: &str, include_query: &Option<IncludeQuery>, fields_query: &Option<FieldsQuery>,
    ) -> RbhOptionRes<Included> {
        self.as_slice().included(uri, include_query, fields_query)
    }

    fn to_document_automatically(&self, uri: &str, query: &Query) -> RbhOptionRes<Document> {
        self.as_slice().to_document_automatically(uri, &query)
    }
}
