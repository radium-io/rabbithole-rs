use crate::entity::SingleEntity;
use crate::model::document::Document;
use crate::model::query::Query;
use crate::model::relationship::Relationship;

use crate::model::link::RawUri;
use async_trait::async_trait;

#[async_trait]
pub trait Fetching {
    type Error;
    type Item: SingleEntity;

    /// User defined `vec_to_document` function
    async fn vec_to_document(
        items: &[Self::Item], uri: &str, query: &Query, request_path: &RawUri,
    ) -> Result<Document, Self::Error>;
    /// Mapping to `/<ty>?<query>`
    async fn fetch_collection(query: &Query) -> Result<Vec<Self::Item>, Self::Error>;
    /// Mapping to `/<ty>/<id>?<query>`
    async fn fetch_single(id: &str, query: &Query) -> Result<Option<Self::Item>, Self::Error>;
    /// Mapping to `/<ty>/<id>/relationships/<related_field>?<query>`
    async fn fetch_relationship(
        id: &str, related_field: &str, uri: &str, query: &Query, request_path: &RawUri,
    ) -> Result<Relationship, Self::Error>;
    /// Mapping to `/<ty>/<id>/<related_field>?<query>`
    async fn fetch_related(
        id: &str, related_field: &str, uri: &str, query: &Query, request_path: &RawUri,
    ) -> Result<Document, Self::Error>;
}
