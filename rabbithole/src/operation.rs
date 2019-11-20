use crate::entity::{Entity, SingleEntity};
use crate::model::document::Document;
use crate::model::relationship::Relationship;

use crate::model::error;
use crate::model::link::RawUri;
use crate::model::resource::{IdentifierData, Resource};
use crate::query::Query;
use crate::RbhResult;
use async_trait::async_trait;

pub trait Operation {
    type Item: SingleEntity + Send + Sync;
}

#[async_trait]
pub trait Fetching: Operation {
    /// User defined `vec_to_document` function
    /// NOTICE:
    ///   - If using Page Query, it's *recommended* to:
    ///     - put `prev`, `next`, `first` and `last` into `links`
    ///     - put `totalPages` if `@type == PageBased`
    async fn vec_to_document(
        items: &[Self::Item], uri: &str, query: &Query, request_path: &RawUri,
    ) -> Result<Document, error::Error> {
        Ok(items.to_document_automatically(uri, query, request_path)?)
    }
    /// Mapping to `/<ty>?<query>`
    #[allow(unused_variables)]
    async fn fetch_collection(&self, query: &Query) -> Result<Vec<Self::Item>, error::Error> {
        Err(error::Error::OperationNotImplemented("fetch_collection", None))
    }
    /// Mapping to `/<ty>/<id>?<query>`
    #[allow(unused_variables)]
    async fn fetch_single(
        &self, id: &str, query: &Query,
    ) -> Result<Option<Self::Item>, error::Error> {
        Err(error::Error::OperationNotImplemented("fetch_single", None))
    }
    /// Mapping to `/<ty>/<id>/relationships/<related_field>?<query>`
    #[allow(unused_variables)]
    async fn fetch_relationship(
        &self, id: &str, related_field: &str, uri: &str, query: &Query, request_path: &RawUri,
    ) -> Result<Relationship, error::Error> {
        Err(error::Error::OperationNotImplemented("fetch_relationship", None))
    }
    /// Mapping to `/<ty>/<id>/<related_field>?<query>`
    #[allow(unused_variables)]
    async fn fetch_related(
        &self, id: &str, related_field: &str, uri: &str, query: &Query, request_path: &RawUri,
    ) -> Result<Document, error::Error> {
        Err(error::Error::OperationNotImplemented("fetch_related", None))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ResourceDataWrapper {
    pub data: Resource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdentifierDataWrapper {
    pub data: IdentifierData,
}

#[async_trait]
pub trait Creating: Operation {
    /// Mapping to `POST /<ty>`
    #[allow(unused_variables)]
    async fn create(&mut self, data: &ResourceDataWrapper) -> RbhResult<Self::Item> {
        Err(error::Error::OperationNotImplemented("create", None))
    }
}

#[async_trait]
pub trait Updating: Operation {
    /// Mapping to `PATCH /<ty>/<id>`
    #[allow(unused_variables)]
    async fn update_resource(
        &mut self, id: &str, data: &ResourceDataWrapper,
    ) -> RbhResult<Self::Item> {
        Err(error::Error::OperationNotImplemented("update_resource", None))
    }
    /// Mapping to `PATCH /<ty>/<id>/relationships/<field>`
    #[allow(unused_variables)]
    async fn replace_relationship(
        &mut self, id_field: &(String, String), data: &IdentifierDataWrapper,
    ) -> RbhResult<Self::Item> {
        Err(error::Error::OperationNotImplemented("replace_relationship", None))
    }
    /// Mapping to `POST /<ty>/<id>/relationships/<field>`
    #[allow(unused_variables)]
    async fn add_relationship(
        &mut self, id_field: &(String, String), data: &IdentifierDataWrapper,
    ) -> RbhResult<Self::Item> {
        Err(error::Error::OperationNotImplemented("add_relationship", None))
    }
    /// Mapping to `DELETE /<ty>/<id>/relationships/<field>`
    #[allow(unused_variables)]
    async fn remove_relationship(
        &mut self, id_field: &(String, String), data: &IdentifierDataWrapper,
    ) -> RbhResult<Self::Item> {
        Err(error::Error::OperationNotImplemented("remove_relationship", None))
    }
}

#[async_trait]
pub trait Deleting: Operation {
    /// Mapping to `DELETE /<ty>/<id>`
    #[allow(unused_variables)]
    async fn delete_resource(&mut self, id: &str) -> RbhResult<()> {
        Err(error::Error::OperationNotImplemented("delete_resource", None))
    }
}
