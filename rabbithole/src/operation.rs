use crate::entity::{Entity, SingleEntity};
use crate::model::document::Document;
use crate::model::relationship::Relationship;

use crate::model::error;
use crate::model::link::RawUri;
use crate::query::Query;
use async_trait::async_trait;

#[async_trait]
pub trait Fetching {
    type Item: SingleEntity + Send + Sync;

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
    async fn fetch_collection(query: &Query) -> Result<Vec<Self::Item>, error::Error>;
    /// Mapping to `/<ty>/<id>?<query>`
    async fn fetch_single(id: &str, query: &Query) -> Result<Option<Self::Item>, error::Error>;
    /// Mapping to `/<ty>/<id>/relationships/<related_field>?<query>`
    async fn fetch_relationship(
        id: &str, related_field: &str, uri: &str, query: &Query, request_path: &RawUri,
    ) -> Result<Relationship, error::Error>;
    /// Mapping to `/<ty>/<id>/<related_field>?<query>`
    async fn fetch_related(
        id: &str, related_field: &str, uri: &str, query: &Query, request_path: &RawUri,
    ) -> Result<serde_json::Value, error::Error>;
}
