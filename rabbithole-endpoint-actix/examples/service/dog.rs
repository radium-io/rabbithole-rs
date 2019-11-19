use async_trait::async_trait;

use rabbithole::operation::*;

use rabbithole::model::error;
use rabbithole::model::error::Error;

use crate::model::dog::Dog;
use crate::service::*;
use log::info;
use rabbithole::model::resource::AttributeField;
use rabbithole::query::Query;

use futures::lock::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Default)]
pub struct DogService(HashMap<String, Dog>);
impl DogService {
    pub fn new() -> Arc<Mutex<Self>> { Arc::new(Mutex::new(Default::default())) }

    pub fn get_by_id(&self, id: &str) -> Option<Dog> { self.0.get(id).cloned() }

    pub fn get_by_ids(&self, ids: &[String]) -> Result<Vec<Dog>, error::Error> {
        let res: Result<Vec<Dog>, error::Error> = ids
            .iter()
            .map(|id| {
                if let Some(dog) = self.0.get(id) {
                    Ok(dog.clone())
                } else {
                    Err(INVALID_IDS_CONTAINED.clone())
                }
            })
            .collect();
        Ok(res?)
    }
}

impl Operation for DogService {
    type Item = Dog;
}

#[async_trait]
impl Fetching for DogService {
    async fn fetch_collection(&self, _query: &Query) -> Result<Vec<Dog>, Error> {
        info!("fetching_all: {}", self.0.len());
        Ok(self.0.values().cloned().collect())
    }

    async fn fetch_single(&self, id: &str, _query: &Query) -> Result<Option<Dog>, Error> {
        Ok(self.0.get(id).map(Clone::clone))
    }
}
#[async_trait]
impl Creating for DogService {
    async fn create(&mut self, data: &ResourceDataWrapper) -> Result<Dog, Error> {
        let ResourceDataWrapper { data } = data;
        if let AttributeField(serde_json::Value::String(name)) =
            data.attributes.get_field("name")?
        {
            let dog = Dog { id: Uuid::new_v4(), name: name.clone() };
            info!("=== data: {:?}", dog);
            info!("map before creating: {}", self.0.len());
            self.0.insert(dog.id.clone().to_string(), dog.clone());
            info!("map after creating: {}", self.0.len());
            Ok(dog)
        } else {
            Err(WRONG_FIELD_TYPE.clone())
        }
    }
}
#[async_trait]
impl Updating for DogService {}
#[async_trait]
impl Deleting for DogService {}
