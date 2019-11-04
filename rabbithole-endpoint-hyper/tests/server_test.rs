#![feature(async_closure)]

#[macro_use]
extern crate rabbithole_derive;
extern crate rabbithole_derive as rbh_derive;

use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use serde::{Deserialize, Serialize};
use tokio_test::block_on;
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

#[test]
fn server_start_test() {
    type GenericError = String;
    type Result<T> = std::result::Result<T, GenericError>;

    let uri_path_prefix = "/api/v1";

    let addr = ([127, 0, 0, 1], 3000).into();

    let human_service = async move |_: Request<Body>| -> Result<Response<Body>> {
        Ok(Response::new(Body::from("human_service")))
    };

    let dog_service = async move |_: Request<Body>| -> Result<Response<Body>> {
        Ok(Response::new(Body::from("dog_service")))
    };

    let services = async move |req: Request<Body>| {
        if req.uri().path() == format!("{}/people", uri_path_prefix) {
            human_service(req).await
        } else if req.uri().path() == format!("{}/dogs", uri_path_prefix) {
            dog_service(req).await
        } else {
            Err("Not found route".into())
        }
    };

    let service = make_service_fn(async move |_| {
        Ok::<_, GenericError>(service_fn(async move |req| services(req).await))
    });
    let server = Server::bind(&addr).serve(service);

    if let Err(err) = block_on(server) {
        unreachable!("error happened: {}", err);
    }
}
