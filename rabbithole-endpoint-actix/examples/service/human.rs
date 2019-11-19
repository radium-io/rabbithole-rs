use async_trait::async_trait;

use rabbithole::operation::*;

use rabbithole::model::error;
use rabbithole::model::error::Error;

use crate::model::dog::Dog;
use crate::model::human::Human;
use crate::service::dog::DogService;
use rabbithole::model::resource::{AttributeField, ResourceIdentifier};
use rabbithole::query::Query;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct HumanService(pub HashMap<String, Human>, pub Arc<Mutex<DogService>>);

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
        let dog_ids: Vec<&str> = dog_ids.iter().map(|id| id.id.as_str()).collect();
        let dogs: Vec<Dog> = self.1.lock().unwrap().get_by_ids(&dog_ids)?;

        if let AttributeField(serde_json::Value::String(name)) =
            data.attributes.get_field("name")?
        {
            let human = Human { id: Uuid::new_v4(), name: name.clone(), dogs };
            self.0.insert(human.id.clone().to_string(), human.clone());
            Ok(human)
        } else {
            Err(error::Error {
                status: Some("400".into()),
                code: Some("1".into()),
                title: Some("Wrong field type".into()),
                ..Default::default()
            })
        }
    }
}
#[async_trait]
impl Updating for HumanService {}
#[async_trait]
impl Deleting for HumanService {}
