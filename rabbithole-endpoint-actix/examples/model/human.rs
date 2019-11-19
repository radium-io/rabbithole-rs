extern crate rabbithole_derive as rbh_derive;
use crate::model::dog::Dog;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone, Debug)]
#[entity(type = "humans")]
#[entity(service(crate::service::human::HumanService))]
#[entity(backend(actix))]
pub struct Human {
    #[entity(id)]
    pub id: Uuid,
    pub name: String,
    #[entity(to_many)]
    pub dogs: Vec<Dog>,
}
