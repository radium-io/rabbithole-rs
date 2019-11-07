use crate::model::document::{Document, Included};
use crate::model::link::{Link, Links, RawUri};
use crate::model::query::{FieldsQuery, IncludeQuery, Query};
use crate::model::relationship::{RelationshipLinks, Relationships};
use crate::model::resource::{Attributes, Resource, ResourceIdentifier};
use crate::{RbhOptionRes, RbhResult};
use serde::Serialize;

use std::collections::HashMap;
use std::iter::FromIterator;

pub trait SingleEntity: Entity {
    #[doc(hidden)]
    fn ty() -> String;
    #[doc(hidden)]
    fn id(&self) -> String;
    #[doc(hidden)]
    fn attributes(&self) -> Attributes;
    #[doc(hidden)]
    fn relationships(&self, uri: &str) -> RbhResult<Relationships>;

    #[doc(hidden)]
    fn links(&self, uri: &str) -> RbhResult<Links> {
        let slf = format!(
            "{uri}/{ty}/{id}",
            uri = uri,
            ty = <Self as SingleEntity>::ty(),
            id = self.id()
        )
        .parse::<Link>()?;
        Ok(HashMap::from_iter(vec![("self".into(), slf)]))
    }

    fn to_document_automatically(
        &self, uri: &str, query: &Query, request_path: &RawUri,
    ) -> RbhResult<Document> {
        Ok(Document::single_resource(
            self.to_resource(uri, &query.fields)?.unwrap(),
            self.included(uri, &query.include, &query.fields)?,
            Some(HashMap::from_iter(vec![Link::slf(uri, request_path.clone())])),
        ))
    }

    fn to_resource_identifier(&self) -> Option<ResourceIdentifier> {
        Some(ResourceIdentifier { ty: <Self as SingleEntity>::ty(), id: self.id() })
    }

    fn to_resource(&self, uri: &str, fields_query: &FieldsQuery) -> RbhOptionRes<Resource> {
        let mut attributes = self.attributes();
        let mut relationships = self.relationships(uri)?;
        for (k, vs) in fields_query.iter() {
            if &<Self as SingleEntity>::ty() == k {
                attributes = attributes.retain(vs);
                relationships.retain(|k, _| vs.contains(k));
            }
        }

        Ok(Some(Resource {
            ty: <Self as SingleEntity>::ty(),
            id: self.id(),
            attributes,
            relationships,
            links: self.links(uri)?,
            ..Default::default()
        }))
    }

    fn to_relationship_links(&self, field_name: &str, uri: &str) -> RbhResult<RelationshipLinks> {
        let slf = format!(
            "{uri}/{ty}/{id}/relationships/{field_name}",
            uri = uri,
            ty = <Self as SingleEntity>::ty(),
            id = self.id(),
            field_name = field_name
        );
        let slf = slf.parse::<Link>()?;
        let related = format!(
            "{uri}/{ty}/{id}/{field_name}",
            uri = uri,
            ty = <Self as SingleEntity>::ty(),
            id = self.id(),
            field_name = field_name
        );
        let related = related.parse::<Link>()?;
        let links: RelationshipLinks =
            HashMap::from_iter(vec![("self".into(), slf), ("related".into(), related)]).into();
        Ok(links)
    }
}

pub trait Entity: Serialize {
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
        &self, uri: &str, include_query: &Option<IncludeQuery>, fields_query: &FieldsQuery,
    ) -> RbhResult<Included>;

    /// Returns a `Document` based on `query`. This function will do all of the actions databases should do in memory,
    /// using a trivial iter way. But I still recommend you guys implement `to_document` or `to_document_async` yourself
    /// for better performance
    fn to_document_automatically(
        &self, uri: &str, query: &Query, request_path: &RawUri,
    ) -> RbhResult<Document>;
}

impl<T: SingleEntity> SingleEntity for Option<T> {
    fn ty() -> String { T::ty() }

    fn id(&self) -> String { self.as_ref().map(SingleEntity::id).unwrap() }

    fn attributes(&self) -> Attributes { self.as_ref().map(SingleEntity::attributes).unwrap() }

    fn relationships(&self, uri: &str) -> RbhResult<Relationships> {
        self.as_ref().map(|op| op.relationships(uri)).unwrap()
    }

    fn to_document_automatically(
        &self, uri: &str, query: &Query, request_path: &RawUri,
    ) -> RbhResult<Document> {
        if let Some(item) = self {
            SingleEntity::to_document_automatically(item, uri, query, request_path)
        } else {
            Ok(Document::none())
        }
    }

    fn to_resource_identifier(&self) -> Option<ResourceIdentifier> { None }

    fn to_resource(&self, _: &str, _: &FieldsQuery) -> RbhOptionRes<Resource> { Ok(None) }
}

impl<T: Entity> Entity for Option<T> {
    fn included(
        &self, uri: &str, include_query: &Option<IncludeQuery>, fields_query: &FieldsQuery,
    ) -> RbhResult<Included> {
        if let Some(s) = self {
            s.included(uri, include_query, fields_query)
        } else {
            Ok(Default::default())
        }
    }

    fn to_document_automatically(
        &self, uri: &str, query: &Query, request_path: &RawUri,
    ) -> RbhResult<Document> {
        self.as_ref().map(|op| op.to_document_automatically(uri, query, request_path)).unwrap()
    }
}

impl<T: SingleEntity> SingleEntity for Box<T> {
    fn ty() -> String { T::ty() }

    fn id(&self) -> String { self.as_ref().id() }

    fn attributes(&self) -> Attributes { self.as_ref().attributes() }

    fn relationships(&self, uri: &str) -> RbhResult<Relationships> {
        self.as_ref().relationships(uri)
    }
}

impl<T: Entity> Entity for Box<T> {
    fn included(
        &self, uri: &str, include_query: &Option<IncludeQuery>, fields_query: &FieldsQuery,
    ) -> RbhResult<Included> {
        self.as_ref().included(uri, include_query, fields_query)
    }

    fn to_document_automatically(
        &self, uri: &str, query: &Query, request_path: &RawUri,
    ) -> RbhResult<Document> {
        self.as_ref().to_document_automatically(uri, query, request_path)
    }
}

impl<T: SingleEntity> SingleEntity for &T {
    fn ty() -> String { T::ty() }

    fn id(&self) -> String { (*self).id() }

    fn attributes(&self) -> Attributes { (*self).attributes() }

    fn relationships(&self, uri: &str) -> RbhResult<Relationships> { (*self).relationships(uri) }
}

impl<T: Entity> Entity for &T {
    fn included(
        &self, uri: &str, include_query: &Option<IncludeQuery>, fields_query: &FieldsQuery,
    ) -> RbhResult<Included> {
        (*self).included(uri, include_query, fields_query)
    }

    fn to_document_automatically(
        &self, uri: &str, query: &Query, request_path: &RawUri,
    ) -> RbhResult<Document> {
        (*self).to_document_automatically(uri, query, request_path)
    }
}

impl<T: SingleEntity> Entity for [T] {
    fn included(
        &self, uri: &str, include_query: &Option<IncludeQuery>, fields_query: &FieldsQuery,
    ) -> RbhResult<Included> {
        let mut hashmap: HashMap<ResourceIdentifier, Resource> = Default::default();
        for e in self {
            for i in e.included(uri, include_query, fields_query)? {
                hashmap.insert(ResourceIdentifier { ty: i.ty.clone(), id: i.id.clone() }, i);
            }
        }

        Ok(hashmap.values().cloned().collect())
    }

    #[cfg(feature = "unstable-vec-to-document")]
    fn to_document_automatically(
        &self, uri: &str, query: &Query, request_path: &RawUri,
    ) -> RbhResult<Document> {
        let mut reses = vec![];
        for e in self {
            if let Some(res) = e.to_resource(uri, &query.fields)? {
                reses.push(res);
            }
        }
        Ok(Document::multiple_resources(
            reses,
            self.included(uri, &query.include, &query.fields)?,
            Some(HashMap::from_iter(vec![Link::slf(uri, request_path.clone().into())])),
        ))
    }

    #[cfg(not(feature = "unstable-vec-to-document"))]
    fn to_document_automatically(&self, _: &str, _: &Query, _: &RawUri) -> RbhResult<Document> {
        unimplemented!()
    }
}

impl<T: SingleEntity> Entity for Vec<T> {
    fn included(
        &self, uri: &str, include_query: &Option<IncludeQuery>, fields_query: &FieldsQuery,
    ) -> RbhResult<Included> {
        self.as_slice().included(uri, include_query, fields_query)
    }

    #[cfg(feature = "unstable-vec-to-document")]
    fn to_document_automatically(
        &self, uri: &str, query: &Query, request_path: &RawUri,
    ) -> RbhResult<Document> {
        self.as_slice().to_document_automatically(uri, &query, request_path)
    }

    #[cfg(not(feature = "unstable-vec-to-document"))]
    fn to_document_automatically(&self, _: &str, _: &Query, _: &RawUri) -> RbhResult<Document> {
        unimplemented!()
    }
}
