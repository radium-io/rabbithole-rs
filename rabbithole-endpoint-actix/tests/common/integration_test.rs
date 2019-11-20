use actix_web::http::StatusCode;
use actix_web::{test, web};

use crate::{init_app, send_request};

use actix_web::body::Body;
use actix_web::dev::{Service, ServiceResponse};

use crate::model::dog::generate_dogs;
use crate::model::human::Human;
use rabbithole::model::document::{Document, DocumentItem};
use rabbithole::model::resource::{AttributeField, Attributes};

use rabbithole::operation::ResourceDataWrapper;
use std::collections::HashMap;
use std::iter::FromIterator;

fn resp_to_doc(mut resp: ServiceResponse) -> Document {
    if let Some(Body::Bytes(ref bytes)) = resp.take_body().as_ref() {
        let body = String::from_utf8(Vec::from(bytes.as_ref())).unwrap();
        serde_json::from_str(&body).unwrap()
    } else {
        unreachable!();
    }
}

#[test]
fn test() {
    let (host, path, mut app) = init_app!(1, 0);
    // When field type not found
    let resp: ServiceResponse = send_request!(app, get, "{}/humans/1", path);
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // When entity not found
    let resp: ServiceResponse = send_request!(app, get, "{}/people/1", path);
    assert!(resp.status().is_success());
    let doc = resp_to_doc(resp);
    if let DocumentItem::PrimaryData(None) = doc.item {
    } else {
        unreachable!("Entity should not be found");
    }

    // Add some dogs
    let dogs = generate_dogs(5);
    let dog_resources = ResourceDataWrapper::from_entities(&dogs, &host);
    for dog in &dog_resources {
        let resp: ServiceResponse = send_request!(app, post, dog, "{}/dogs", path);
        assert!(resp.status().is_success());
        let doc = resp_to_doc(resp);
        let (resource, _) = doc.into_single().unwrap();
        assert_eq!(resource.id, dog.data.id);
    }

    // Get all dogs
    let resp: ServiceResponse = send_request!(app, get, "{}/dogs", path);
    assert!(resp.status().is_success());
    let doc = resp_to_doc(resp);
    let (new_dog_resources, _) = doc.into_multiple().unwrap();
    assert_eq!(new_dog_resources.len(), dog_resources.len());
    let new_dog_resource = new_dog_resources.first().unwrap();
    let old_matched_resource =
        &dog_resources.iter().find(|r| r.data.id == new_dog_resource.id).unwrap().data;
    assert_eq!(new_dog_resource, old_matched_resource);

    // Update the name of the first dog
    let mut first_dog_res = dog_resources.first().cloned().unwrap();
    let first_dog_id = &first_dog_res.data.id.id;
    let new_name = serde_json::Value::String("new_name_changed".into());
    let map: HashMap<String, serde_json::Value> =
        HashMap::from_iter(vec![("name".into(), new_name.clone())]);
    let map: Attributes = map.into();
    first_dog_res.data.attributes = map;
    let resp: ServiceResponse =
        send_request!(app, patch, first_dog_res, "{}/dogs/{}", path, first_dog_id);
    assert!(resp.status().is_success());
    let doc = resp_to_doc(resp);
    let (resource, _) = doc.into_single().unwrap();
    let AttributeField(name) = resource.attributes.get_field("name").unwrap();
    assert_eq!(name, &new_name);

    // Add some humans
    let humans: Vec<Human> = dogs.windows(2).map(Human::from).collect();
    let human_resources = ResourceDataWrapper::from_entities(&humans, &host);
    for human in human_resources {
        let resp: ServiceResponse = send_request!(app, post, human, "{}/people", path);
        assert!(resp.status().is_success());
        let doc = resp_to_doc(resp);
        let (resource, _) = doc.into_single().unwrap();
        assert_eq!(resource.id, human.data.id);
        assert!(resource.relationships.contains_key("dogs"));
        let dogs_relat = resource.relationships.get("dogs").unwrap();
        let dogs_relat = dogs_relat.data.data();
        assert_eq!(dogs_relat.len(), 2);
    }
}
