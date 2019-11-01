extern crate rabbithole_derive as rbh_derive;
extern crate serde;

use rabbithole::entity::Entity;
use rabbithole::entity::EntityExt;
use rabbithole::error::RabbitholeError;
use rabbithole::model::document::{Document, Included};
use rabbithole::model::link::Link;
use rabbithole::model::relationship::Relationship;
use rabbithole::model::resource::{Attributes, IdentifierData, Resource, ResourceIdentifier};
use rabbithole::RbhResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::mem::transmute;

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
#[entity(type = "humans")]
pub struct Human {
    #[entity(id)]
    pub passport_number: String,
    pub name: String,
    #[entity(to_one)]
    pub only_flea: Box<Flea>,
    pub gender: Gender,
}

#[derive(Serialize, Deserialize, Clone)]
#[entity(type = "dogs")]
pub struct Dog {
    #[entity(id)]
    pub id: String,
    pub name: String,
    #[entity(to_many(Flea))]
    pub fleas: Vec<Flea>,
    #[entity(to_many)]
    pub friends: Vec<Dog>,
    #[entity(to_one)]
    pub master: Box<Human>,
    #[entity(to_one(Dog))]
    pub best_one: Option<Box<Dog>>,
}

impl Entity for Dog {
    fn ty(&self) -> String { unimplemented!() }

    fn id(&self) -> String { unimplemented!() }

    fn attributes(&self) -> Attributes { unimplemented!() }

    fn relationships(
        &self, uri: &str,
    ) -> Result<HashMap<String, Relationship, RandomState>, RabbitholeError> {
        unimplemented!()
    }

    fn included(&self, uri: &str) -> Result<Vec<Resource>, RabbitholeError> {
        let mut included: Included = Default::default();

        impl EntityExt<Dog> for Option<Box<Dog>> {
            fn to_entity(&self) -> Option<&Dog> { self.as_deref() }
        }

        if let Some(field) = &self.best_one.to_entity() {
            included.push(field.to_resource(uri)?);
        }
        if let Some(&field) = self.master.to_entity() {
            included.push(field.to_resource(uri)?);
        }

        Ok(included)
    }
}

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
#[entity(type = "fleas")]
pub struct Flea {
    #[entity(id)]
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Gender {
    Male,
    Female,
    Unknown,
}

#[test]
fn test() {
    let master_flea = Flea { id: "1".to_string(), name: "master_flea".to_string() };

    let _master_flea_res = Resource {
        ty: "fleas".to_string(),
        id: "1".to_string(),
        attributes: HashMap::from_iter(vec![("name".into(), Value::String("master_flea".into()))])
            .into(),
        relationships: Default::default(),
        links: HashMap::from_iter(vec![(
            "self".into(),
            "https://example.com/api/fleas/1".parse::<Link>().unwrap(),
        )]),
        ..Default::default()
    };

    let master = Human {
        passport_number: "number".to_string(),
        name: "master_name".to_string(),
        only_flea: Box::new(master_flea),
        gender: Gender::Male,
    };

    let master_res = Resource {
        ty: "humans".to_string(),
        id: "number".to_string(),
        attributes: HashMap::from_iter(vec![
            ("name".into(), Value::String("master_name".into())),
            ("gender".into(), Value::String("Male".into())),
        ])
        .into(),
        relationships: HashMap::from_iter(vec![("only_flea".into(), Relationship {
            data: IdentifierData::Single(Some(ResourceIdentifier {
                ty: "fleas".to_string(),
                id: "1".to_string(),
            })),
            links: HashMap::from_iter(vec![
                (
                    "self".into(),
                    "https://example.com/api/humans/number/relationships/only_flea"
                        .parse::<Link>()
                        .unwrap(),
                ),
                (
                    "related".into(),
                    "https://example.com/api/humans/number/only_flea".parse::<Link>().unwrap(),
                ),
            ])
            .into(),
            ..Default::default()
        })]),
        links: HashMap::from_iter(vec![(
            "self".into(),
            "https://example.com/api/humans/number".parse::<Link>().unwrap(),
        )]),
        ..Default::default()
    };

    let dog_flea_a = Flea { id: "a".to_string(), name: "dog_flea_a".to_string() };

    let dog_flea_a_res = Resource {
        ty: "fleas".to_string(),
        id: "a".to_string(),
        attributes: HashMap::from_iter(vec![("name".into(), Value::String("dog_flea_a".into()))])
            .into(),
        links: HashMap::from_iter(vec![(
            "self".into(),
            "https://example.com/api/fleas/a".parse::<Link>().unwrap(),
        )]),
        ..Default::default()
    };

    let dog_flea_b = Flea { id: "b".to_string(), name: "dog_flea_b".to_string() };

    let dog_flea_b_res = Resource {
        ty: "fleas".to_string(),
        id: "b".to_string(),
        attributes: HashMap::from_iter(vec![("name".into(), Value::String("dog_flea_b".into()))])
            .into(),
        links: HashMap::from_iter(vec![(
            "self".into(),
            "https://example.com/api/fleas/b".parse::<Link>().unwrap(),
        )]),
        ..Default::default()
    };

    let dog = Dog {
        id: "1".to_string(),
        name: "dog_name".to_string(),
        fleas: vec![dog_flea_a, dog_flea_b],
        friends: vec![],
        master: Box::new(master),
        best_one: None,
    };

    let dog_res = Resource {
        ty: "dogs".to_string(),
        id: "1".to_string(),
        attributes: HashMap::from_iter(vec![("name".into(), Value::String("dog_name".into()))])
            .into(),
        relationships: HashMap::from_iter(vec![
            ("friends".into(), Relationship {
                data: IdentifierData::Multiple(Default::default()),
                links: HashMap::from_iter(vec![
                    (
                        "self".into(),
                        "https://example.com/api/dogs/1/relationships/friends"
                            .parse::<Link>()
                            .unwrap(),
                    ),
                    (
                        "related".into(),
                        "https://example.com/api/dogs/1/friends".parse::<Link>().unwrap(),
                    ),
                ])
                .into(),
                meta: Default::default(),
            }),
            ("fleas".into(), Relationship {
                data: IdentifierData::Multiple(vec![
                    ResourceIdentifier { ty: "fleas".to_string(), id: "a".to_string() },
                    ResourceIdentifier { ty: "fleas".to_string(), id: "b".to_string() },
                ]),
                links: HashMap::from_iter(vec![
                    (
                        "self".into(),
                        "https://example.com/api/dogs/1/relationships/fleas"
                            .parse::<Link>()
                            .unwrap(),
                    ),
                    (
                        "related".into(),
                        "https://example.com/api/dogs/1/fleas".parse::<Link>().unwrap(),
                    ),
                ])
                .into(),
                ..Default::default()
            }),
            ("master".into(), Relationship {
                data: IdentifierData::Single(Some(ResourceIdentifier {
                    ty: "humans".to_string(),
                    id: "number".to_string(),
                })),
                links: HashMap::from_iter(vec![
                    (
                        "self".into(),
                        "https://example.com/api/dogs/1/relationships/master"
                            .parse::<Link>()
                            .unwrap(),
                    ),
                    (
                        "related".into(),
                        "https://example.com/api/dogs/1/master".parse::<Link>().unwrap(),
                    ),
                ])
                .into(),
                ..Default::default()
            }),
        ]),
        links: HashMap::from_iter(vec![(
            "self".into(),
            "https://example.com/api/dogs/1".parse::<Link>().unwrap(),
        )]),
        ..Default::default()
    };

    let document =
        Document::single_resource(dog_res, vec![master_res, dog_flea_a_res, dog_flea_b_res]);

    let gen_doc: Document = dog.to_document("https://example.com/api").unwrap();
    assert_eq!(document, gen_doc);
}
