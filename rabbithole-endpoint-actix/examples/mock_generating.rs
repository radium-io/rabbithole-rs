extern crate rabbithole_derive as rbh_derive;

use actix_web::App;
use actix_web::{middleware, web};
use actix_web::{HttpResponse, HttpServer};
use async_trait::async_trait;

use rabbithole::entity::{Entity, SingleEntity};
use rabbithole::model::error;
use rabbithole::model::link::RawUri;
use rabbithole::model::relationship::Relationship;
use rabbithole::operation::{Creating, Deleting, Fetching, Operation, Updating};
use rabbithole::query::Query;
use rabbithole_endpoint_actix::settings::ActixSettingsModel;
use rabbithole_endpoint_actix::ActixSettings;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::sync::Mutex;
use uuid::Uuid;

// rbh_derive::EntityDecorator to generate JSON:API data models
#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
// JSON:API Resource type
#[entity(type = "people")]
// Generate actix backend
#[entity(backend(actix))]
#[entity(service(HumanService))]
pub struct Human {
    #[entity(id)]
    pub id_code: Uuid,
    pub name: String,
    #[entity(to_many)]
    pub dogs: Vec<Dog>,
}

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
#[entity(type = "dogs")]
#[entity(backend(actix))]
#[entity(service(DogService))]
pub struct Dog {
    #[entity(id)]
    pub id: Uuid,
    pub name: String,
}

impl From<&[Dog]> for Human {
    fn from(dogs: &[Dog]) -> Self {
        let uuid = Uuid::new_v4();
        Self { id_code: uuid, name: uuid.to_string(), dogs: dogs.to_vec() }
    }
}

fn generate_dogs(len: usize) -> Vec<Dog> {
    let mut dogs = Vec::with_capacity(len);
    for _ in 0 .. len {
        let uuid = Uuid::new_v4();
        dogs.push(Dog { id: uuid, name: uuid.to_string() });
    }
    dogs
}

fn generate_masters(len: usize) -> Vec<Human> {
    let mut masters = Vec::with_capacity(len);
    for i in 0 ..= len {
        let uuid = Uuid::new_v4();
        let dogs = generate_dogs(i);
        masters.push(Human { id_code: uuid, name: uuid.to_string(), dogs });
    }
    masters
}

#[derive(Default)]
pub struct DogService();
impl Operation for DogService {
    type Item = Dog;
}

#[async_trait]
impl Creating for DogService {}
#[async_trait]
impl Updating for DogService {}
#[async_trait]
impl Deleting for DogService {}

#[async_trait]
impl Fetching for DogService {
    async fn fetch_collection(&self, _query: &Query) -> Result<Vec<Dog>, error::Error> {
        let rand = rand::random::<usize>() % 5;
        let dogs = generate_dogs(rand);
        Ok(dogs)
    }

    async fn fetch_single(&self, id: &str, _query: &Query) -> Result<Option<Dog>, error::Error> {
        if id == "none" {
            Ok(None)
        } else {
            let rand = rand::random::<usize>() % 3;
            Ok(generate_dogs(rand).first().cloned())
        }
    }

    async fn fetch_relationship(
        &self, _: &str, related_field: &str, _: &str, _: &Query, _: &RawUri,
    ) -> Result<Relationship, error::Error> {
        Err(error::Error::FieldNotExist(related_field, None))
    }

    async fn fetch_related(
        &self, _: &str, related_field: &str, _: &str, _: &Query, _: &RawUri,
    ) -> Result<serde_json::Value, error::Error> {
        Err(error::Error::FieldNotExist(related_field, None))
    }
}

#[derive(Default)]
pub struct HumanService();
impl Operation for HumanService {
    type Item = Human;
}

#[async_trait]
impl Creating for HumanService {}
#[async_trait]
impl Updating for HumanService {}
#[async_trait]
impl Deleting for HumanService {}

#[async_trait]
impl Fetching for HumanService {
    async fn fetch_collection(&self, _: &Query) -> Result<Vec<Human>, error::Error> {
        let rand = rand::random::<usize>() % 5 + 1;
        let masters = generate_masters(rand);
        Ok(masters)
    }

    async fn fetch_single(&self, id: &str, _query: &Query) -> Result<Option<Human>, error::Error> {
        if id == "none" {
            Ok(None)
        } else {
            let rand = rand::random::<usize>() % 3 + 1;
            Ok(generate_masters(rand).first().cloned())
        }
    }

    async fn fetch_relationship(
        &self, id: &str, related_field: &str, uri: &str, _query: &Query, _request_path: &RawUri,
    ) -> Result<Relationship, error::Error> {
        if related_field == "dogs" {
            if id == "none" {
                return Err(error::Error::ParentResourceNotExist(related_field, None));
            }

            let rand = rand::random::<usize>() % 3;
            let relats = generate_masters(rand).last().cloned().unwrap().relationships(uri);
            Ok(relats.get(related_field).cloned().unwrap())
        } else {
            Err(error::Error::FieldNotExist(related_field, None))
        }
    }

    async fn fetch_related(
        &self, id: &str, related_field: &str, uri: &str, query: &Query, request_path: &RawUri,
    ) -> Result<serde_json::Value, error::Error> {
        if related_field == "dogs" {
            if id == "none" {
                return Err(error::Error::ParentResourceNotExist(related_field, None));
            }

            let rand = rand::random::<usize>() % 3;
            let master = generate_masters(rand);
            let master = master.last().unwrap();
            serde_json::to_value(master.dogs.to_document_automatically(uri, query, request_path))
                .map_err(|err| error::Error::InvalidJson(&err, None))
        } else {
            Err(error::Error::FieldNotExist(related_field, None))
        }
    }
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("config/actix.config.example.toml")).unwrap();
    let settings: ActixSettingsModel = settings.try_into().unwrap();
    let settings_port = settings.port;

    HttpServer::new(move || {
        App::new()
            .register_data(web::Data::new(Mutex::new(DogService::default())))
            .register_data(web::Data::new(Mutex::new(HumanService::default())))
            .data::<ActixSettings<DogService>>(settings.clone().try_into().unwrap())
            .data::<ActixSettings<HumanService>>(settings.clone().try_into().unwrap())
            .wrap(middleware::Logger::new(r#"%a "%r" %s %b "%{Referer}i" "%{Content-Type}i" %T"#))
            .service(
                web::scope(&settings.path)
                    .service(DogService::actix_service())
                    .service(HumanService::actix_service()),
            )
            .default_service(web::to(HttpResponse::NotFound))
    })
    .bind(format!("[::]:{}", settings_port))?
    .run()
}
