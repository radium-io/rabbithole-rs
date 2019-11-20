use crate::init_app;
use actix_web::dev::Service;
use actix_web::dev::ServiceResponse;
use actix_web::http::{header, StatusCode};
use actix_web::{test, web};
use rabbithole::JSON_API_HEADER;

#[test]
/// https://jsonapi.org/format/#content-negotiation-servers
fn invalid_accept_header_test() {
    let (_, path, mut app) = init_app!(1, 0);
    let req = test::TestRequest::get()
        .uri(&format!("{}/people/1", path))
        .header(header::CONTENT_TYPE, JSON_API_HEADER)
        .to_request();
    let future = test::run_on(|| app.call(req));
    let resp: ServiceResponse = test::block_on(future).unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE);
}

#[test]
/// https://jsonapi.org/format/#content-negotiation-servers
fn invalid_content_type_test() {
    let (_, path, mut app) = init_app!(1, 0);
    let req = test::TestRequest::get()
        .uri(&format!("{}/people/1", path))
        .header(header::ACCEPT, JSON_API_HEADER)
        .to_request();
    let future = test::run_on(|| app.call(req));
    let resp: ServiceResponse = test::block_on(future).unwrap();
    assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[test]
/// https://jsonapi.org/format/#content-negotiation-servers
fn invalid_content_type_params_test() {
    let (_, path, mut app) = init_app!(1, 0);
    let req = test::TestRequest::get()
        .uri(&format!("{}/people/1", path))
        .header(header::ACCEPT, JSON_API_HEADER)
        .header(
            header::CONTENT_TYPE,
            format!(r#"{}; profile="cursor-pagination""#, JSON_API_HEADER),
        )
        .to_request();
    let future = test::run_on(|| app.call(req));
    let resp: ServiceResponse = test::block_on(future).unwrap();
    assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}
