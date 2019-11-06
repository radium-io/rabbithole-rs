#![feature(core_intrinsics)]

use rabbithole::model::document::Document;
extern crate rabbithole_derive as rbh_derive;

use actix_web::http::StatusCode;
use actix_web::{web, App};
use actix_web::{HttpResponse, HttpServer};
use async_trait::async_trait;

use rabbithole::entity::{Entity, SingleEntity};
use rabbithole::model::query::Query;
use rabbithole::model::relationship::Relationship;

use rabbithole::model::Id;
use rabbithole::operation::Fetching;
use rabbithole_endpoint_actix::ActixSettings;

use serde::{Deserialize, Serialize};

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
    for i in 0 .. len + 1 {
        let uuid = Uuid::new_v4();
        let dogs = generate_dogs(i);
        masters.push(Human { id_code: uuid, name: uuid.to_string(), dogs });
    }
    masters
}

#[async_trait]
impl Fetching for Human {
    type Error = HttpResponse;
    type Item = Human;

    async fn vec_to_document(
        items: &[Self::Item], uri: &str, query: &Query,
    ) -> Result<Document, Self::Error> {
        if let Ok(doc) = items.to_document_automatically(uri, query) {
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

    async fn fetch_single(id: &Id, _query: &Query) -> Result<Option<Self::Item>, Self::Error> {
        if id == "none" {
            Ok(None)
        } else {
            let rand = rand::random::<usize>() % 3;
            Ok(generate_masters(rand).first().cloned())
        }
    }

    async fn fetch_relationship(
        _id: &Id, related_field: &str, uri: &str, _query: &Query,
    ) -> Result<Relationship, Self::Error> {
        let rand = rand::random::<usize>() % 3;
        if let Ok(relats) = generate_masters(rand).last().cloned().unwrap().relationships(uri) {
            Ok(relats.get(related_field).cloned().unwrap())
        } else {
            Err(HttpResponse::build(StatusCode::BAD_REQUEST).body("error"))
        }
    }

    async fn fetch_related(
        _id: &Id, related_field: &str, uri: &str, query: &Query,
    ) -> Result<Document, Self::Error> {
        if related_field == "dogs" {
            let rand = rand::random::<usize>() % 3;
            if let Some(master) = generate_masters(rand).last() {
                if let Ok(doc) = master.dogs.to_document_automatically(uri, query) {
                    Ok(doc)
                } else {
                    Err(HttpResponse::build(StatusCode::BAD_REQUEST).body("doc parsing error"))
                }
            } else {
                unreachable!()
            }
        } else {
            Err(HttpResponse::build(StatusCode::BAD_REQUEST).body("unhandled related fields"))
        }
    }
}

fn print_type_of<T>(_: &T) {
    println!("{}", unsafe { std::intrinsics::type_name::<T>() });
}
fn main() -> std::io::Result<()> {
    let server = HttpServer::new(move || {
        App::new()
            .data(ActixSettings::<Human>::new("http://example.com/api/v1"))
            .route(
                "/api/v1/people",
                web::get().to_async(move |req, actix_fetching: web::Data<ActixSettings<Human>>| {
                    actix_fetching.get_ref().clone().fetch_collection(req)
                }),
            )
            .route(
                "/api/v1/people/{id}",
                web::get().to_async(
                    move |param, req, actix_fetching: web::Data<ActixSettings<Human>>| {
                        actix_fetching.get_ref().clone().fetch_single(param, req)
                    },
                ),
            )
            .route(
                "/api/v1/people/{id}/relationships/{related_fields}",
                web::get().to_async(
                    move |param, req, actix_fetching: web::Data<ActixSettings<Human>>| {
                        actix_fetching.get_ref().clone().fetch_relationship(param, req)
                    },
                ),
            )
            .route(
                "/api/v1/people/{id}/{related_fields}",
                web::get().to_async(
                    move |param, req, actix_fetching: web::Data<ActixSettings<Human>>| {
                        actix_fetching.get_ref().clone().fetch_related(param, req)
                    },
                ),
            )
    });

    print_type_of(&server);
    Ok(())
}
