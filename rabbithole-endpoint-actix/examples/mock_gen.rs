extern crate rabbithole_derive as rbh_derive;

use actix_web::http::StatusCode;
use actix_web::App;
use actix_web::{middleware, web};
use actix_web::{HttpResponse, HttpServer};
use async_trait::async_trait;
use rabbithole::model::document::Document;

use rabbithole::entity::{Entity, SingleEntity};
use rabbithole::model::query::Query;
use rabbithole::model::relationship::Relationship;

use rabbithole::operation::Fetching;

use serde::{Deserialize, Serialize};

use rabbithole::model::link::{Link, RawUri};
use rabbithole_endpoint_actix::settings::ActixSettingsModel;
use rabbithole_endpoint_actix::ActixSettings;
use std::collections::HashMap;
use std::convert::TryInto;
use std::iter::FromIterator;
use uuid::Uuid;

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
#[entity(type = "people")]
#[entity(backend(actix))]
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

#[async_trait]
impl Fetching for Dog {
    type Error = HttpResponse;
    type Item = Dog;

    async fn vec_to_document(
        items: &[Self::Item], uri: &str, query: &Query, request_path: &RawUri,
    ) -> Result<Document, Self::Error> {
        if let Ok(doc) = items.to_document_automatically(uri, query, request_path) {
            Ok(doc)
        } else {
            Err(HttpResponse::build(StatusCode::BAD_REQUEST).body("error"))
        }
    }

    async fn fetch_collection(_query: &Query) -> Result<Vec<Self::Item>, Self::Error> {
        let rand = rand::random::<usize>() % 5;
        let dogs = generate_dogs(rand);
        Ok(dogs)
    }

    async fn fetch_single(id: &str, _query: &Query) -> Result<Option<Self::Item>, Self::Error> {
        if id == "none" {
            Ok(None)
        } else {
            let rand = rand::random::<usize>() % 3;
            Ok(generate_dogs(rand).first().cloned())
        }
    }

    async fn fetch_relationship(
        _id: &str, _related_field: &str, uri: &str, _query: &Query, request_path: &RawUri,
    ) -> Result<Relationship, Self::Error> {
        Err(HttpResponse::Ok().json(Document::null(Some(HashMap::from_iter(vec![Link::slf(
            uri,
            request_path.clone(),
        )])))))
    }

    async fn fetch_related(
        _id: &str, _related_field: &str, uri: &str, _query: &Query, request_path: &RawUri,
    ) -> Result<Document, Self::Error> {
        Err(HttpResponse::Ok().json(Document::null(Some(HashMap::from_iter(vec![Link::slf(
            uri,
            request_path.clone(),
        )])))))
    }
}

#[async_trait]
impl Fetching for Human {
    type Error = HttpResponse;
    type Item = Human;

    async fn vec_to_document(
        items: &[Self::Item], uri: &str, query: &Query, request_path: &RawUri,
    ) -> Result<Document, Self::Error> {
        if let Ok(doc) = items.to_document_automatically(uri, query, request_path) {
            Ok(doc)
        } else {
            Err(HttpResponse::build(StatusCode::BAD_REQUEST).body("error"))
        }
    }

    async fn fetch_collection(_: &Query) -> Result<Vec<Self::Item>, Self::Error> {
        let rand = rand::random::<usize>() % 5;
        let masters = generate_masters(rand);
        Ok(masters)
    }

    async fn fetch_single(id: &str, _query: &Query) -> Result<Option<Self::Item>, Self::Error> {
        if id == "none" {
            Ok(None)
        } else {
            let rand = rand::random::<usize>() % 3;
            Ok(generate_masters(rand).first().cloned())
        }
    }

    async fn fetch_relationship(
        _id: &str, related_field: &str, uri: &str, _query: &Query, _request_path: &RawUri,
    ) -> Result<Relationship, Self::Error> {
        if related_field == "dogs" {
            let rand = rand::random::<usize>() % 3;
            let relats = generate_masters(rand).last().cloned().unwrap().relationships(uri);
            Ok(relats.get(related_field).cloned().unwrap())
        } else {
            Err(HttpResponse::NotFound().finish())
        }
    }

    async fn fetch_related(
        _id: &str, related_field: &str, uri: &str, query: &Query, request_path: &RawUri,
    ) -> Result<Document, Self::Error> {
        if related_field == "dogs" {
            let rand = rand::random::<usize>() % 3;
            let master = generate_masters(rand);
            let master = master.last().unwrap();
            if let Ok(doc) = master.dogs.to_document_automatically(uri, query, request_path) {
                Ok(doc)
            } else {
                Err(HttpResponse::InternalServerError().body("doc parsing error"))
            }
        } else {
            Err(HttpResponse::NotFound().finish())
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
            .data::<ActixSettings<Human>>(settings.clone().try_into().unwrap())
            .data::<ActixSettings<Dog>>(settings.clone().try_into().unwrap())
            .wrap(middleware::Logger::new(r#"%a "%r" %s %b "%{Referer}i" "%{Content-Type}i" %T"#))
            .service(
                web::scope(&settings.path)
                    .service(Human::actix_service())
                    .service(Dog::actix_service()),
            )
            .default_service(web::to(HttpResponse::NotFound))
    })
    .bind(format!("[::]:{}", settings_port))?
    .run()
}
