use actix_web::http::{header, StatusCode};
use actix_web::{test, web};

use crate::{classes_init, fetching_init, init_app};

use actix_web::body::Body;
use actix_web::dev::{Service, ServiceResponse};
use rabbithole::model::document::{Document, DocumentItem, PrimaryDataItem};
use rabbithole::model::error;
use rabbithole::JSON_API_HEADER;
use serde_json::Value;

classes_init!();
fetching_init!();

#[test]
fn single_primary_master_test() {
    let (path, mut app) = init_app!(1, 0);
    let req = test::TestRequest::get()
        .uri(&format!("{}/people/1", path))
        .header(header::CONTENT_TYPE, JSON_API_HEADER)
        .header(header::ACCEPT, JSON_API_HEADER)
        .to_request();
    let future = test::run_on(|| app.call(req));
    let mut resp: ServiceResponse = test::block_on(future).unwrap();
    assert!(resp.status().is_success());

    if let Some(Body::Bytes(ref bytes)) = resp.take_body().as_ref() {
        let body = String::from_utf8(Vec::from(bytes.as_ref())).unwrap();
        let body: Document = serde_json::from_str(&body).unwrap();
        if let DocumentItem::PrimaryData(Some((PrimaryDataItem::Single(resource), _))) = body.item {
            assert_eq!(resource.id.ty, "people");
            assert!(resource.relationships.contains_key("dogs"));
        } else {
            unreachable!("Expect single primary data");
        }
    } else {
        unreachable!();
    }
}

#[test]
fn none_master_test() {
    let (path, mut app) = init_app!(1, 0);
    let req = test::TestRequest::get()
        .uri(&format!("{}/people/none", path))
        .header(header::CONTENT_TYPE, JSON_API_HEADER)
        .header(header::ACCEPT, JSON_API_HEADER)
        .to_request();
    let future = test::run_on(|| app.call(req));
    let mut resp: ServiceResponse = test::block_on(future).unwrap();
    assert!(resp.status().is_success());

    if let Some(Body::Bytes(ref bytes)) = resp.take_body().as_ref() {
        let body = String::from_utf8(Vec::from(bytes.as_ref())).unwrap();
        let body: Value = serde_json::from_str(&body).unwrap();
        assert!(body.get("data").is_some());
        assert!(body.get("data").unwrap().is_null());

        let body: Document = serde_json::from_value(body).unwrap();
        if let DocumentItem::PrimaryData(None) = body.item {
        } else {
            unreachable!("Expect None data");
        }
    } else {
        unreachable!();
    }
}

#[test]
fn single_primary_master_collection_test() {
    let (path, mut app) = init_app!(1, 0);
    let req = test::TestRequest::get()
        .uri(&format!("{}/people", path))
        .header(header::CONTENT_TYPE, JSON_API_HEADER)
        .header(header::ACCEPT, JSON_API_HEADER)
        .to_request();
    let future = test::run_on(|| app.call(req));
    let mut resp: ServiceResponse = test::block_on(future).unwrap();
    assert!(resp.status().is_success());

    if let Some(Body::Bytes(ref bytes)) = resp.take_body().as_ref() {
        let body = String::from_utf8(Vec::from(bytes.as_ref())).unwrap();
        let body: Document = serde_json::from_str(&body).unwrap();
        if let DocumentItem::PrimaryData(Some((PrimaryDataItem::Multiple(resource), _))) = body.item
        {
            assert!(!resource.is_empty());
            assert!(resource.first().is_some());
            assert_eq!(resource.first().unwrap().id.ty, "people");
        } else {
            unreachable!("Expect primary data array");
        }
    } else {
        unreachable!();
    }
}

#[test]
fn related_dogs_test() {
    let (path, mut app) = init_app!(1, 0);
    let req = test::TestRequest::get()
        .uri(&format!("{}/people/1/dogs", path))
        .header(header::CONTENT_TYPE, JSON_API_HEADER)
        .header(header::ACCEPT, JSON_API_HEADER)
        .to_request();
    let future = test::run_on(|| app.call(req));
    let resp = test::block_on(future);
    let mut resp: ServiceResponse = resp.unwrap();
    assert!(resp.status().is_success());

    if let Some(Body::Bytes(ref bytes)) = resp.take_body().as_ref() {
        let body = String::from_utf8(Vec::from(bytes.as_ref())).unwrap();
        let body: Document = serde_json::from_str(&body).unwrap();
        if let DocumentItem::PrimaryData(Some((PrimaryDataItem::Multiple(resources), _))) =
            body.item
        {
            assert!(!resources.is_empty());
            assert!(resources.first().is_some());
            assert_eq!(resources.first().unwrap().id.ty, "dogs");
        } else {
            unreachable!("Expect primary data array");
        }
    } else {
        unreachable!();
    }
}

#[test]
fn empty_dogs_test() {
    let (path, mut app) = init_app!(1, 0);
    let req = test::TestRequest::get()
        .uri(&format!("{}/dogs", path))
        .header(header::CONTENT_TYPE, JSON_API_HEADER)
        .header(header::ACCEPT, JSON_API_HEADER)
        .to_request();
    let future = test::run_on(|| app.call(req));
    let mut resp: ServiceResponse = test::block_on(future).unwrap();
    assert!(resp.status().is_success());

    if let Some(Body::Bytes(ref bytes)) = resp.take_body().as_ref() {
        let body = String::from_utf8(Vec::from(bytes.as_ref())).unwrap();
        let body: Value = serde_json::from_str(&body).unwrap();
        assert!(body.get("data").is_some());
        assert!(body.get("data").unwrap().as_array().unwrap().is_empty());
        let body: Document = serde_json::from_value(body).unwrap();
        if let DocumentItem::PrimaryData(Some((PrimaryDataItem::Multiple(resources), _))) =
            body.item
        {
            assert!(resources.is_empty());
        } else {
            unreachable!("Expect empty array");
        }
    } else {
        unreachable!();
    }
}

#[test]
fn no_master_dog_test() {
    let (path, mut app) = init_app!(1, 0);
    let req = test::TestRequest::get()
        .uri(&format!("{}/people/none/dogs", path))
        .header(header::CONTENT_TYPE, JSON_API_HEADER)
        .header(header::ACCEPT, JSON_API_HEADER)
        .to_request();
    let future = test::run_on(|| app.call(req));
    let mut resp: ServiceResponse = test::block_on(future).unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    if let Some(Body::Bytes(body)) = resp.take_body().as_ref() {
        let body = String::from_utf8(Vec::from(body.as_ref())).unwrap();
        let body: error::Error = serde_json::from_str(&body).unwrap();
        assert_eq!(body.status, Some("404".into()));
        assert_eq!(body.code, Some("RBH-0404".into()));
        assert_eq!(body.title, Some("Parent Resource of Relationship Not Exist".into()));
    }
}

#[test]
fn relationship_with_no_existing_fields_test() {
    let (path, mut app) = init_app!(1, 0);
    let req = test::TestRequest::get()
        .uri(&format!("{}/people/1/relationships/no_exist", path))
        .header(header::CONTENT_TYPE, JSON_API_HEADER)
        .header(header::ACCEPT, JSON_API_HEADER)
        .to_request();
    let future = test::run_on(|| app.call(req));
    let mut resp: ServiceResponse = test::block_on(future).unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    if let Some(Body::Bytes(body)) = resp.take_body().as_ref() {
        let body = String::from_utf8(Vec::from(body.as_ref())).unwrap();
        let body: error::Error = serde_json::from_str(&body).unwrap();
        assert_eq!(body.status, Some("404".into()));
        assert_eq!(body.code, Some("RBH-0401".into()));
        assert_eq!(body.title, Some("Field Not Exist".into()));
        assert_eq!(body.detail, Some("Field `no_exist` does not exist".into()));
    }
}

#[test]
fn include_nothing_test() {
    let (path, mut app) = init_app!(1, 0);
    let req = test::TestRequest::get()
        .uri(&format!("{}/people/1?include=", path))
        .header(header::CONTENT_TYPE, JSON_API_HEADER)
        .header(header::ACCEPT, JSON_API_HEADER)
        .to_request();
    let future = test::run_on(|| app.call(req));
    let mut resp: ServiceResponse = test::block_on(future).unwrap();
    assert!(resp.status().is_success());

    if let Some(Body::Bytes(body)) = resp.take_body().as_ref() {
        let body = String::from_utf8(Vec::from(body.as_ref())).unwrap();
        let body: Document = serde_json::from_str(&body).unwrap();
        if let DocumentItem::PrimaryData(Some((PrimaryDataItem::Single(resource), included))) =
            body.item
        {
            assert!(included.is_empty());
            assert_eq!(resource.id.ty, "people");
            assert!(resource.relationships.get("dogs").is_some());
            assert!(resource.attributes.get_field("name").is_ok());
        } else {
            unreachable!();
        }
    }
}

#[test]
fn only_name_field_test() {
    let (path, mut app) = init_app!(1, 0);
    let req = test::TestRequest::get()
        .uri(&format!("{}/people/1?fields%5Bpeople%5D=name", path))
        .header(header::CONTENT_TYPE, JSON_API_HEADER)
        .header(header::ACCEPT, JSON_API_HEADER)
        .to_request();
    let future = test::run_on(|| app.call(req));
    let mut resp: ServiceResponse = test::block_on(future).unwrap();
    //        assert!(resp.status().is_success());

    if let Some(Body::Bytes(body)) = resp.take_body().as_ref() {
        let body = String::from_utf8(Vec::from(body.as_ref())).unwrap();
        let body: Document = serde_json::from_str(&body).unwrap();
        if let DocumentItem::PrimaryData(Some((PrimaryDataItem::Single(resource), included))) =
            body.item
        {
            assert!(!included.is_empty());
            assert_eq!(resource.id.ty, "people");
            assert!(resource.relationships.get("dogs").is_none());
            assert!(resource.attributes.get_field("name").is_ok());
        } else {
            unreachable!();
        }
    }
}

#[test]
fn only_dogs_field_test() {
    let (path, mut app) = init_app!(1, 0);
    let req = test::TestRequest::get()
        .uri(&format!("{}/people/1?fields%5Bpeople%5D=dogs", path))
        .header(header::CONTENT_TYPE, JSON_API_HEADER)
        .header(header::ACCEPT, JSON_API_HEADER)
        .to_request();
    let future = test::run_on(|| app.call(req));
    let mut resp: ServiceResponse = test::block_on(future).unwrap();
    assert!(resp.status().is_success());

    if let Some(Body::Bytes(body)) = resp.take_body().as_ref() {
        let body = String::from_utf8(Vec::from(body.as_ref())).unwrap();
        let body: Document = serde_json::from_str(&body).unwrap();
        if let DocumentItem::PrimaryData(Some((PrimaryDataItem::Single(resource), included))) =
            body.item
        {
            assert!(!included.is_empty());
            assert_eq!(resource.id.ty, "people");
            assert!(resource.relationships.get("dogs").is_some());
            assert!(resource.attributes.get_field("name").is_err());
        } else {
            unreachable!();
        }
    }
}
