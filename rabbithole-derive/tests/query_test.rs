extern crate rabbithole_derive as rbh_derive;
extern crate serde;

use rabbithole::entity::{Entity, SingleEntity};
use rabbithole::model::document::{Document, Included};
use rabbithole::model::link::Link;
use rabbithole::model::resource::Resource;
use rabbithole::query::Query;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::iter::FromIterator;
use uuid::Uuid;

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
#[entity(type = "people")]
#[entity(service(HumanService))]
pub struct Human {
    #[entity(id)]
    pub id_code: Uuid,
    pub name: String,
    #[entity(to_many)]
    pub dogs: Vec<Dog>,
}

impl From<&[Dog]> for Human {
    fn from(dogs: &[Dog]) -> Self {
        let uuid = Uuid::new_v4();
        Self {
            id_code: uuid,
            name: uuid.to_string(),
            dogs: dogs.to_vec(),
        }
    }
}

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
#[entity(type = "dogs")]
#[entity(service(DogService))]
pub struct Dog {
    #[entity(id)]
    pub id: Uuid,
    pub name: String,
}

fn generate_dogs(len: usize) -> Vec<Dog> {
    let mut dogs = Vec::with_capacity(len);
    for _ in 0 .. len {
        let uuid = Uuid::new_v4();
        dogs.push(Dog {
            id: uuid,
            name: uuid.to_string(),
        });
    }
    dogs
}

fn generate_masters() -> Vec<Human> {
    let master1_dogs = generate_dogs(2);
    let master1: Human = master1_dogs.as_slice().into();

    let master2_dogs = generate_dogs(3);
    let master2: Human = master2_dogs.as_slice().into();

    vec![master1, master2]
}

#[test]
fn default_include_test() {
    let uri = "https://example.com/api";

    let master_vec = generate_masters();
    let gen_doc = master_vec.to_document(
        "https://example.com/api",
        &Default::default(),
        uri.parse().unwrap(),
        Default::default(),
        Default::default(),
    );

    let master_reses: Vec<Resource> = master_vec
        .iter()
        .map(|h| h.to_resource(uri, &Default::default()).unwrap())
        .collect();

    let mut manual_included: Included = Default::default();
    for m in master_vec {
        for d in m.dogs {
            let d_res: Resource = d
                .to_resource(uri, &Default::default())
                .unwrap()
                .try_into()
                .unwrap();
            manual_included.insert(d_res.id.clone(), d_res);
        }
    }
    let mut manual_doc = Document::multiple_resources(master_reses, manual_included);
    manual_doc.extend_links(HashMap::from_iter(vec![Link::slf(
        "https://example.com",
        "/api".parse::<http::Uri>().unwrap(),
    )]));
    assert_eq!(gen_doc.unwrap(), manual_doc);
}

#[test]
fn only_unknown_include_test() {
    let uri = "https://example.com/api";

    let master_vec = generate_masters();
    let gen_doc = master_vec.to_document(
        "https://example.com/api",
        &Query {
            include: Some(HashSet::from_iter(vec!["name".to_string()])),
            ..Default::default()
        },
        uri.parse().unwrap(),
        Default::default(),
        Default::default(),
    );

    let master_reses: Vec<Resource> = master_vec
        .iter()
        .map(|h| h.to_resource(uri, &Default::default()).unwrap())
        .collect();
    let mut manual_doc = Document::multiple_resources(master_reses, Default::default());
    manual_doc.extend_links(HashMap::from_iter(vec![Link::slf(
        "https://example.com",
        "/api".parse::<http::Uri>().unwrap(),
    )]));
    assert_eq!(gen_doc.unwrap(), manual_doc);
}

#[test]
fn not_included_fields_but_retain_attributes() {
    let uri = "https://example.com/api";

    let master_vec = generate_masters();
    let gen_doc = master_vec.to_document(
        "https://example.com/api",
        &Query {
            include: Some(Default::default()),
            ..Default::default()
        },
        uri.parse().unwrap(),
        Default::default(),
        Default::default(),
    );

    let master_reses: Vec<Resource> = master_vec
        .iter()
        .map(|h| h.to_resource(uri, &Default::default()).unwrap())
        .collect();
    let mut manual_doc = Document::multiple_resources(master_reses, Default::default());
    manual_doc.extend_links(HashMap::from_iter(vec![Link::slf(
        "https://example.com",
        "/api".parse::<http::Uri>().unwrap(),
    )]));
    assert_eq!(gen_doc.unwrap(), manual_doc);
}

#[test]
fn not_foreign_attributes_but_retain_included_fields() {
    let uri = "https://example.com/api";
    let fields_query = HashMap::from_iter(vec![(
        "people".into(),
        HashSet::from_iter(vec!["name".into()]),
    )]);

    let master_vec = generate_masters();
    let gen_doc = master_vec.to_document(
        "https://example.com/api",
        &Query {
            fields: fields_query.clone(),
            ..Default::default()
        },
        uri.parse().unwrap(),
        Default::default(),
        Default::default(),
    );

    let master_reses: Vec<Resource> = master_vec
        .iter()
        .map(|h| h.to_resource(uri, &fields_query).unwrap())
        .collect();
    let mut manual_included: Included = Default::default();
    for m in master_vec {
        for d in m.dogs {
            let d_res: Resource = d
                .to_resource(uri, &Default::default())
                .unwrap()
                .try_into()
                .unwrap();
            manual_included.insert(d_res.id.clone(), d_res);
        }
    }
    let mut manual_doc = Document::multiple_resources(master_reses, manual_included);
    manual_doc.extend_links(HashMap::from_iter(vec![Link::slf(
        "https://example.com",
        "/api".parse::<http::Uri>().unwrap(),
    )]));
    assert_eq!(gen_doc.unwrap(), manual_doc);
}
