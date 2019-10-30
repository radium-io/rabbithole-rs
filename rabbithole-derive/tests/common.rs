#[macro_use]
extern crate rabbithole_derive as rbh_derive;

use rabbithole::model::document::{Document, DocumentItem, PrimaryDataItem};
use rabbithole::model::link::{Link, RawUri};
use rabbithole::model::relationship::{Relationship, RelationshipLinks};
use rabbithole::model::resource::{Attributes, IdentifierData, Resource, ResourceIdentifier};
use serde_json::Value;
use std::collections::HashMap;
use std::iter::FromIterator;

#[derive(rbh_derive::ModelDecorator)]
#[model(type = "dogs")]
pub struct Dog {
    #[model(id)]
    pub id: String,
    pub name: String,
    #[model(to_many)]
    pub fleas: Vec<Flea>,
    pub master: Human,
}

#[derive(rbh_derive::ModelDecorator)]
#[model(type = "fleas")]
pub struct Flea {
    #[model(id)]
    pub id: String,
    pub name: String,
}

#[derive(rbh_derive::ModelDecorator)]
#[model(type = "humans")]
pub struct Human {
    #[model(id)]
    pub passport_number: String,
    pub name: String,
    #[model(to_one)]
    pub only_flea: Option<Flea>,
    pub gender: Gender,
}

pub enum Gender {
    Male,
    Female,
    Unknown,
}

#[test]
fn test() {
    let master_flea = Flea {
        id: "1".to_string(),
        name: "master_flea".to_string(),
    };

    let master_flea_res = Resource {
        ty: "fleas".to_string(),
        id: "1".to_string(),
        attributes: HashMap::from_iter(vec![("name".into(), Value::String("master_flea".into()))])
            .into(),
        relationships: Default::default(),
        links: HashMap::from_iter(vec![(
            "self".into(),
            "https://example.com/api/fleas/1".parse::<Link>().unwrap(),
        )]),
        meta: Default::default(),
    };

    let master = Human {
        passport_number: "number".to_string(),
        name: "master_name".to_string(),
        only_flea: Some(master_flea),
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
        relationships: HashMap::from_iter(vec![(
            "only_flea".into(),
            Relationship {
                data: IdentifierData::Multiple(vec![ResourceIdentifier {
                    ty: "fleas".to_string(),
                    id: "1".to_string(),
                }]),
                links: HashMap::from_iter(vec![
                    (
                        "self".into(),
                        "https://example.com/api/humans/number/relationships/only_flea"
                            .parse::<Link>()
                            .unwrap(),
                    ),
                    (
                        "related".into(),
                        "https://example.com/api/humans/number/only_flea"
                            .parse::<Link>()
                            .unwrap(),
                    ),
                ])
                .into(),
                meta: Default::default(),
            },
        )]),
        links: HashMap::from_iter(vec![(
            "self".into(),
            "https://example.com/api/humans/number"
                .parse::<Link>()
                .unwrap(),
        )]),
        meta: Default::default(),
    };

    let dog_flea_a = Flea {
        id: "a".to_string(),
        name: "dog_flea_a".to_string(),
    };

    let dog_flea_a_res = Resource {
        ty: "fleas".to_string(),
        id: "a".to_string(),
        attributes: HashMap::from_iter(vec![("name".into(), Value::String("dog_flea_a".into()))])
            .into(),
        relationships: Default::default(),
        links: HashMap::from_iter(vec![(
            "self".into(),
            "https://example.com/api/fleas/a".parse::<Link>().unwrap(),
        )]),
        meta: Default::default(),
    };

    let dog_flea_b = Flea {
        id: "b".to_string(),
        name: "dog_flea_b".to_string(),
    };

    let dog_flea_b_res = Resource {
        ty: "fleas".to_string(),
        id: "b".to_string(),
        attributes: HashMap::from_iter(vec![("name".into(), Value::String("dog_flea_b".into()))])
            .into(),
        relationships: Default::default(),
        links: HashMap::from_iter(vec![(
            "self".into(),
            "https://example.com/api/fleas/b".parse::<Link>().unwrap(),
        )]),
        meta: Default::default(),
    };

    let dog = Dog {
        id: "1".to_string(),
        name: "dog_name".to_string(),
        fleas: vec![dog_flea_a, dog_flea_b],
        master,
    };

    let dog_res = Resource {
        ty: "dogs".to_string(),
        id: "1".to_string(),
        attributes: HashMap::from_iter(vec![("name".into(), Value::String("dog_name".into()))])
            .into(),
        relationships: HashMap::from_iter(vec![
            (
                "fleas".into(),
                Relationship {
                    data: IdentifierData::Multiple(vec![
                        ResourceIdentifier {
                            ty: "fleas".to_string(),
                            id: "a".to_string(),
                        },
                        ResourceIdentifier {
                            ty: "fleas".to_string(),
                            id: "b".to_string(),
                        },
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
                            "https://example.com/api/dogs/1/fleas"
                                .parse::<Link>()
                                .unwrap(),
                        ),
                    ])
                    .into(),
                    meta: Default::default(),
                },
            ),
            (
                "master".into(),
                Relationship {
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
                            "https://example.com/api/dogs/1/master"
                                .parse::<Link>()
                                .unwrap(),
                        ),
                    ])
                    .into(),
                    meta: Default::default(),
                },
            ),
        ]),
        links: HashMap::from_iter(vec![(
            "self".into(),
            "https://example.com/api/dogs/1".parse::<Link>().unwrap(),
        )]),
        meta: Default::default(),
    };

    let document =
        Document::single_resource(dog_res, vec![master_res, dog_flea_a_res, dog_flea_b_res]);

    let json = serde_json::to_string_pretty(&document).unwrap();
    println!("json: {}", json);
}
