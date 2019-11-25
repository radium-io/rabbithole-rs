extern crate rabbithole_derive as rbh_derive;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone, Debug)]
#[entity(type = "dogs")]
#[entity(service(crate::service::dog::DogService))]
#[entity(backend(actix))]
pub struct Dog {
    #[entity(id)]
    pub id: Uuid,
    pub name: String,
}
