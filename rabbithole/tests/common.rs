extern crate rabbithole_derive as rbh_derive;

use serde::{Deserialize, Serialize};

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
#[entity(type = "dogs")]
pub struct Dog {
    #[entity(id)]
    pub id: String,
    pub name: String,
    pub age: i32,
}
