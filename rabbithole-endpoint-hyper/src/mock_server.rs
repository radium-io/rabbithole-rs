#![feature(async_closure)]

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use rabbithole::model::query::Query;

#[tokio::main]
async fn main() {
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
        eprintln!("req.uri: {}", req.uri().to_string());
        let query = Query::from_uri(req.uri(), "CursorBased", "Rsql").unwrap();
        eprintln!("query: {:#?}", query);
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
    if let Err(err) = server.await {
        unreachable!("error happened: {}", err);
    }
}
