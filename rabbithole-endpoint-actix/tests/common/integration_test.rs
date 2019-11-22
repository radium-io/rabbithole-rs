use actix_web::http::StatusCode;
use actix_web::web;

use crate::{init_app, send_request};
use actix_http_test::block_on;

use crate::model::dog::{generate_dogs, Dog};
use crate::model::human::Human;
use rabbithole::model::document::{Document, DocumentItem};
use rabbithole::model::resource::{AttributeField, Attributes, IdentifierData, ResourceIdentifier};

use actix_http::encoding::Decoder;
use actix_http::Payload;
use actix_web::client::ClientResponse;
use rabbithole::operation::{IdentifierDataWrapper, ResourceDataWrapper};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

async fn resp_to_doc(mut resp: ClientResponse<Decoder<Payload>>) -> Document {
    let bytes = resp.body().await.unwrap();
    let body = String::from_utf8(Vec::from(bytes.as_ref())).unwrap();
    serde_json::from_str(&body).unwrap()
}

#[test]
#[allow(clippy::cognitive_complexity)]
fn basic_crud_test() {
    block_on(async {
        let (host, path, app) = init_app!(1, 0);
        // When field type not found
        let resp = send_request!(app, get, "{}/humans/1", path);
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        // When entity not found
        let resp = send_request!(app, get, "{}/people/1", path);
        assert!(resp.status().is_success());
        let doc = resp_to_doc(resp).await;
        if let DocumentItem::PrimaryData(None) = doc.item {
        } else {
            unreachable!("Entity should not be found");
        }

        // Add some dogs
        let dogs = generate_dogs(5);
        let dog_resources = ResourceDataWrapper::from_entities(&dogs, &host);
        for dog in &dog_resources {
            let resp = send_request!(app, post, dog, "{}/dogs", path);
            assert!(resp.status().is_success());
            let doc = resp_to_doc(resp).await;
            let (resource, _) = doc.into_single().unwrap();
            assert_eq!(resource.id, dog.data.id);
        }

        // Update the name of the first dog
        let mut first_dog_res = dog_resources.first().cloned().unwrap();
        let first_dog_id = &first_dog_res.data.id.id;
        let new_name = serde_json::Value::String("new_name_changed".into());
        let map: HashMap<String, serde_json::Value> =
            HashMap::from_iter(vec![("name".into(), new_name.clone())]);
        let map: Attributes = map.into();
        first_dog_res.data.attributes = map;
        let resp = send_request!(app, patch, first_dog_res, "{}/dogs/{}", path, first_dog_id);
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        let resp = send_request!(app, get, "{}/dogs/{}", path, first_dog_id);
        assert!(resp.status().is_success());
        let doc = resp_to_doc(resp).await;
        let (resource, _) = doc.into_single().unwrap();
        let AttributeField(name) = resource.attributes.get_field("name").unwrap();
        assert_eq!(name, &new_name);

        // Get all dogs
        let resp = send_request!(app, get, "{}/dogs", path);
        assert!(resp.status().is_success());
        let doc = resp_to_doc(resp).await;
        let (new_dog_resources, _) = doc.into_multiple().unwrap();
        assert_eq!(new_dog_resources.len(), dog_resources.len());
        let new_dog_resource = new_dog_resources.last().unwrap();
        let old_matched_resource =
            &dog_resources.iter().find(|r| r.data.id == new_dog_resource.id).unwrap().data;
        assert_eq!(new_dog_resource, old_matched_resource);

        // Add some humans
        let humans: Vec<Human> = dogs.windows(2).map(Human::from).collect();
        let human_resources = ResourceDataWrapper::from_entities(&humans, &host);
        for human in &human_resources {
            let resp = send_request!(app, post, human, "{}/people", path);
            assert!(resp.status().is_success());
            let doc = resp_to_doc(resp).await;
            let (resource, _) = doc.into_single().unwrap();
            assert_eq!(resource.id, human.data.id);
            assert!(resource.relationships.contains_key("dogs"));
            let dogs_relat = resource.relationships.get("dogs").unwrap();
            let dogs_relat = dogs_relat.data.data();
            assert_eq!(dogs_relat.len(), 2);
        }

        // Get the first human
        let first_human_id = humans.first().unwrap().id.to_string();
        let resp = send_request!(app, get, "{}/people/{}", path, first_human_id);
        assert!(resp.status().is_success());
        let doc = resp_to_doc(resp).await;
        let (first_human_resource, included) = doc.into_single().unwrap();
        assert_eq!(
            &first_human_resource.as_ref().attributes,
            &human_resources.first().unwrap().data.attributes
        );
        assert_eq!(
            &first_human_resource.as_ref().relationships,
            &human_resources.first().unwrap().data.relationships
        );
        for (pet_key, pet_value) in &included {
            let old_dog = new_dog_resources.iter().find(|wrapper| &wrapper.id == pet_key);
            let old_dog = old_dog.unwrap();
            assert_eq!(&pet_value.attributes, &old_dog.attributes);
        }
    });
}

#[test]
fn relationship_test() {
    block_on(async {
        // Prepare data
        let (host, path, app) = init_app!(1, 0);
        let dogs = generate_dogs(3);
        let dog_resources = ResourceDataWrapper::from_entities(&dogs, &host);
        for dog in &dog_resources {
            let resp = send_request!(app, post, dog, "{}/dogs", path);
            assert!(resp.status().is_success());
        }
        let dogs_idents =
            dog_resources.iter().map(|r| r.data.id.clone()).collect::<Vec<ResourceIdentifier>>();
        let (_first_dogs_idents, second_dogs_idents) = dogs_idents.split_first().unwrap();
        let (first_dogs, second_dogs) = dogs.split_first().unwrap();
        let first_dogs: Vec<Dog> = vec![first_dogs].into_iter().cloned().collect();
        let masters = vec![Human::from(first_dogs.as_slice()), second_dogs.into()];
        let first_master_id = masters.first().unwrap().id.to_string();
        let second_master_id = masters[1].id.to_string();
        let master_resources = ResourceDataWrapper::from_entities(&masters, &host);
        for master in &master_resources {
            let resp = send_request!(app, post, master, "{}/people", path);
            assert!(resp.status().is_success());
        }

        // Delete all dogs from the second master
        let deleted_identifier_data =
            IdentifierDataWrapper { data: IdentifierData::Multiple(Vec::from(second_dogs_idents)) };
        let resp = send_request!(
            app,
            delete,
            deleted_identifier_data,
            "{}/people/{}/relationships/dogs",
            path,
            second_master_id
        );
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        let resp = send_request!(app, get, "{}/people/{}", path, second_master_id);
        let doc = resp_to_doc(resp).await;
        let (second_human_resource, included) = doc.into_single().unwrap();
        assert!(included.is_empty());
        assert!(second_human_resource.relationships.get("dogs").unwrap().data.data().is_empty());

        // Add the removed pets to the first master
        let resp = send_request!(
            app,
            post,
            deleted_identifier_data,
            "{}/people/{}/relationships/dogs",
            path,
            first_master_id
        );
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        let resp = send_request!(app, get, "{}/people/{}", path, first_master_id);
        let doc = resp_to_doc(resp).await;
        let (first_human_resource, included) = doc.into_single().unwrap();
        assert_eq!(included.len(), 3);
        let first_pets: Vec<ResourceIdentifier> =
            first_human_resource.relationships.get("dogs").unwrap().data.data();
        let first_pets_set: HashSet<ResourceIdentifier> = HashSet::from_iter(first_pets);
        assert_eq!(first_pets_set, HashSet::from_iter(dogs_idents));
    });
}
