use crate::entity::SingleEntity;
use crate::model::document::Document;
use crate::model::query::Query;
use crate::model::relationship::Relationship;

use crate::model::resource::Resource;
use crate::model::Id;
use async_trait::async_trait;

#[async_trait]
pub trait Fetching {
    type Item: SingleEntity;
    type Error;

    /// User defined `vec_to_document` function
    async fn vec_to_document(items: &[Self::Item], query: &Query) -> Result<Document, Self::Error>;
    /// Mapping to `/<ty>?<query>`
    async fn fetch_collection(query: &Query) -> Result<Vec<Self::Item>, Self::Error>;
    /// Mapping to `/<ty>/<id>?<query>`
    async fn fetch_single(id: Id, query: &Query) -> Result<Option<Self::Item>, Self::Error>;
    /// Mapping to `/<ty>/<id>/relationships/<related_field>?<query>`
    async fn fetch_relationship(
        id: Id, related_field: &str, query: &Query,
    ) -> Result<Relationship, Self::Error>;
    /// Mapping to `/<ty>/<id>/<related_field>?<query>`
    async fn fetch_related(
        id: Id, related_field: &str, query: &Query,
    ) -> Result<Resource, Self::Error>;
}
