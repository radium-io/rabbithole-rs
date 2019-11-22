use crate::init_app;
use actix_http_test::block_on;

use actix_web::http::{header, StatusCode};
use actix_web::web;
use rabbithole::JSON_API_HEADER;
#[test]
/// https://jsonapi.org/format/1.1/#content-negotiation-servers
fn invalid_accept_header_test() {
    block_on(async {
        let (_, path, app) = init_app!(1, 1);
        let resp = app
            .get(&format!("{}/people/1", path))
            .header(header::CONTENT_TYPE, JSON_API_HEADER)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE);
    });
}

#[test]
/// https://jsonapi.org/format/1.1/#content-negotiation-servers
fn invalid_content_type_test() {
    block_on(async {
        let (_, path, app) = init_app!(1, 1);
        let resp = app
            .get(&format!("{}/people/1", path))
            .header(header::ACCEPT, JSON_API_HEADER)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    });
}

#[test]
/// https://jsonapi.org/format/1.1/#content-negotiation-servers
fn invalid_content_type_params_test() {
    block_on(async {
        let (_, path, app) = init_app!(1, 1);
        let resp = app
            .get(&format!("{}/people/1", path))
            .header(header::ACCEPT, format!(r#"{}; profile="cursor-pagination""#, JSON_API_HEADER))
            .header(
                header::CONTENT_TYPE,
                format!(r#"{}; profile="cursor-pagination""#, JSON_API_HEADER),
            )
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    });
}
