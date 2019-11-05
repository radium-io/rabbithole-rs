use rabbithole::model::document::Document;
extern crate rabbithole_derive as rbh_derive;

use actix_web::body::Body;
use actix_web::http::StatusCode;
use actix_web::{web, App, HttpRequest};
use actix_web::{HttpResponse, HttpServer};
use async_trait::async_trait;
use futures::compat::Future01CompatExt;
use futures::{Future, FutureExt, TryFutureExt};
use rabbithole::entity::{Entity, SingleEntity};
use rabbithole::model::query::Query;
use rabbithole::model::relationship::Relationship;
use rabbithole::model::resource::Resource;
use rabbithole::model::Id;
use rabbithole::operation::Fetching;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    for i in 0 .. len {
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

    async fn vec_to_document(items: &[Self::Item], query: &Query) -> Result<Document, Self::Error> {
        if let Ok(doc) = items.to_document_automatically("http://example.com/api/v1", query) {
            Ok(doc)
        } else {
            Err(HttpResponse::build(StatusCode::BAD_REQUEST).body("error"))
        }
    }

    async fn fetch_collection(_: &Query) -> Result<Vec<Self::Item>, Self::Error> {
        let masters = generate_masters(3);
        Ok(masters)
    }

    async fn fetch_single(id: Id, query: &Query) -> Result<Option<Self::Item>, Self::Error> {
        Ok(generate_masters(1).first().cloned())
    }

    async fn fetch_relationship(
        id: Id, related_field: &str, _query: &Query,
    ) -> Result<Relationship, Self::Error> {
        if let Ok(relats) =
            generate_masters(1).first().cloned().unwrap().relationships("http://example.com/api/v1")
        {
            Ok(relats.get(related_field).cloned().unwrap())
        } else {
            Err(HttpResponse::build(StatusCode::BAD_REQUEST).body("error"))
        }
    }

    async fn fetch_related(
        id: Id, related_field: &str, query: &Query,
    ) -> Result<Resource, Self::Error> {
        Err(HttpResponse::build(StatusCode::BAD_REQUEST).body("error"))
    }
}

fn wrapper_fetch_collection(
    req: HttpRequest,
) -> impl futures01::Future<Item = HttpResponse, Error = actix_web::Error> {
    if let Ok(query) = Query::from_uri(req.uri(), "CursorBased", "Rsql") {
        let fut = async move {
            let vec_res = Human::fetch_collection(&query).await;
            match vec_res {
                //                Ok(vec) => Ok(HttpResponse::Ok().body(format!("size: {}", vec.len()))),
                Ok(vec) => match Human::vec_to_document(&vec, &query).await {
                    Ok(doc) => Ok(HttpResponse::Ok().json(doc)),
                    Err(err) => Ok(err),
                },
                Err(err_resp) => Ok(err_resp),
            }
        };

        fut.boxed_local().compat()
    } else {
        async {
            Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .json(Value::String("Parser query error".into())))
        }
        .boxed_local()
        .compat()
    }
}

fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new().service(
            web::resource("/api/v1/people").route(web::get().to_async(wrapper_fetch_collection)),
        )
    })
    .bind("127.0.0.1:1234")?
    .run()
}
