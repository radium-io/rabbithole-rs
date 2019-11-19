use async_trait::async_trait;

use rabbithole::operation::*;

use rabbithole::model::error::Error;

use crate::model::dog::Dog;
use crate::model::human::Human;
use crate::service::dog::DogService;
use crate::service::*;
use futures::lock::Mutex;
use rabbithole::model::resource::{AttributeField, ResourceIdentifier};
use rabbithole::query::Query;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct HumanService(HashMap<String, Human>, Arc<Mutex<DogService>>);
impl HumanService {
    pub fn new(dog_service: Arc<Mutex<DogService>>) -> Arc<Mutex<HumanService>> {
        Arc::new(Mutex::new(Self(Default::default(), dog_service)))
    }
}

impl Operation for HumanService {
    type Item = Human;
}

#[async_trait]
impl Fetching for HumanService {
    async fn fetch_collection(&self, _query: &Query) -> Result<Vec<Human>, Error> {
        Ok(self.0.values().cloned().collect())
    }

    async fn fetch_single(&self, id: &str, _query: &Query) -> Result<Option<Human>, Error> {
        Ok(self.0.get(id).map(Clone::clone))
    }
}

#[async_trait]
impl Creating for HumanService {
    async fn create(&mut self, data: &ResourceDataWrapper) -> Result<Human, Error> {
        let ResourceDataWrapper { data } = data;
        let dog_ids: Vec<ResourceIdentifier> =
            data.relationships.get("dogs").map(|r| r.data.data()).unwrap_or_default();
        let dog_ids: Vec<String> = dog_ids.iter().map(|id| id.id.clone()).collect();
        let dogs: Vec<Dog> = self.1.lock().await.get_by_ids(&dog_ids)?;

        if let AttributeField(serde_json::Value::String(name)) =
            data.attributes.get_field("name")?
        {
            let human = Human { id: Uuid::new_v4(), name: name.clone(), dogs };
            self.0.insert(human.id.clone().to_string(), human.clone());
            Ok(human)
        } else {
            Err(WRONG_FIELD_TYPE.clone())
        }
    }
}
#[async_trait]
impl Updating for HumanService {
    async fn update_resource(
        &mut self, id: &str, data: &ResourceDataWrapper,
    ) -> Result<Human, Error> {
        if let Some(human) = self.0.get(id) {
            let mut human: Human = human.clone();
            let new_attrs = &data.data.attributes;
            let new_relats = &data.data.relationships;
            if let Some(dog_ids) = new_relats.get("dogs").map(|r| r.data.data()) {
                let dog_ids: Vec<String> =
                    dog_ids.iter().map(|r: &ResourceIdentifier| r.id.clone()).collect();
                let dogs = self.1.clone().lock().await.get_by_ids(&dog_ids)?;
                human.dogs = dogs;
            }
            if let AttributeField(serde_json::Value::String(name)) = new_attrs.get_field("name")? {
                human.name = name.clone();
            }
            self.0.insert(id.to_string(), human.clone());
            Ok(human)
        } else {
            Err(ENTITY_NOT_FOUND.clone())
        }
    }

    async fn replace_relationship(
        &mut self, _id_field: &(String, String), _data: &IdentifierDataWrapper,
    ) -> Result<Human, Error> {
        unimplemented!()
    }

    async fn add_relationship(
        &mut self, _id_field: &(String, String), _data: &IdentifierDataWrapper,
    ) -> Result<Human, Error> {
        unimplemented!()
    }

    async fn remove_relationship(
        &mut self, _id_field: &(String, String), _data: &IdentifierDataWrapper,
    ) -> Result<Human, Error> {
        unimplemented!()
    }
}
#[async_trait]
impl Deleting for HumanService {}
