use async_trait::async_trait;

use rabbithole::operation::*;

use rabbithole::model::error;
use rabbithole::model::error::Error;

use crate::model::dog::Dog;
use crate::service::*;

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
        let id = if !data.id.id.is_empty() {
            if self.0.contains_key(&data.id.id) {
                Err(DUPLICATE_ID.clone())
            } else {
                Uuid::parse_str(&data.id.id).map_err(|_| INVALID_UUID.clone())
            }
        } else {
            Ok(Uuid::new_v4())
        }?;
        if let AttributeField(serde_json::Value::String(name)) =
            data.attributes.get_field("name")?
        {
            let dog = Dog { id, name: name.clone() };
            self.0.insert(dog.id.clone().to_string(), dog.clone());
            Ok(dog)
        } else {
            Err(WRONG_FIELD_TYPE.clone())
        }
    }
}
#[async_trait]
impl Updating for DogService {
    async fn update_resource(
        &mut self, id: &str, data: &ResourceDataWrapper,
    ) -> Result<Dog, Error> {
        if let Some(mut dog) = self.get_by_id(id) {
            let ResourceDataWrapper { data } = data;
            if let AttributeField(serde_json::Value::String(name)) =
                data.attributes.get_field("name")?
            {
                dog.name = name.to_string();
                self.0.insert(id.into(), dog.clone());
                Ok(dog)
            } else {
                Err(WRONG_FIELD_TYPE.clone())
            }
        } else {
            Err(ENTITY_NOT_FOUND.clone())
        }
    }
}
#[async_trait]
impl Deleting for DogService {
    async fn delete_resource(&mut self, id: &str) -> Result<(), Error> {
        self.0.remove(id);
        Ok(())
    }
}
