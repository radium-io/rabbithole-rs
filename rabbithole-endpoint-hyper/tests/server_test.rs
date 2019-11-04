#![feature(async_closure)]
extern crate rabbithole_derive as rbh_derive;
use serde::{Deserialize, Serialize};

use uuid::Uuid;

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
#[entity(type = "people")]
pub struct Human {
    #[entity(id)]
    pub id_code: Uuid,
    pub name: String,
    #[entity(to_many)]
    pub dogs: Vec<Dog>,
}

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
#[entity(type = "dogs")]
pub struct Dog {
    #[entity(id)]
    pub id: Uuid,
    pub name: String,
}
