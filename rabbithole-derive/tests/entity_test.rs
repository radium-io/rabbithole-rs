extern crate rabbithole_derive as rbh_derive;
extern crate serde;

use rabbithole::entity::Entity;
use rabbithole::model::document::{Document, DocumentItem, PrimaryDataItem};
use rabbithole::model::link::{Link};
use rabbithole::model::relationship::Relationship;
use rabbithole::model::resource::*;
use rabbithole::query::Query;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use std::iter::FromIterator;
use uuid::Uuid;

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
#[entity(type = "humans")]
#[entity(service(HumanService))]
pub struct Human {
    #[entity(id)]
    pub passport_number: String,
    pub name: String,
    #[entity(to_one)]
    #[serde(bound(deserialize = "Option<Flea>: Deserialize<'de>"))]
    pub only_flea: Option<Flea>,
    pub gender: Gender,
}

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
#[entity(type = "dogs")]
#[entity(service(DogService))]
pub struct Dog<'a> {
    #[entity(id)]
    pub id: String,
    pub name: String,
    #[entity(to_many)]
    pub fleas: Vec<Flea>,
    #[entity(to_many)]
    pub friends: Vec<Dog<'a>>,
    #[entity(to_one)]
    #[serde(bound(deserialize = "&'a Human: Deserialize<'de>"))]
    pub master: &'a Human,
    #[entity(to_one)]
    pub best_one: Option<Box<Dog<'a>>>,
}

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
#[entity(type = "fleas")]
#[entity(service(FleaService))]
pub struct Flea {
    #[entity(id)]
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Gender {
    Male,
    Female,
    Unknown,
}

impl ToString for Gender {
    fn to_string(&self) -> String {
        match self {
            Gender::Male => "Male".into(),
            Gender::Female => "Female".into(),
            Gender::Unknown => "Unknown".into(),
        }
    }
}

fn generate_fleas(len: usize) -> Vec<Flea> {
    let mut fleas = Vec::with_capacity(len);
    for _i in 0 .. len {
        let uuid = Uuid::new_v4().to_string();
        fleas.push(Flea { id: uuid.clone(), name: uuid.clone() });
    }
    fleas
}

fn generate_dogs(len: usize, master: &Human) -> Vec<Dog> {
    let mut dogs = Vec::with_capacity(len);
    for _i in 0 .. len {
        let fleas = generate_fleas(3);
        let uuid = Uuid::new_v4().to_string();
        dogs.push(Dog {
            id: uuid.clone(),
            name: uuid.clone(),
            fleas,
            friends: vec![],
            master,
            best_one: None,
        });
    }

    dogs
}

fn generate_masters(len: usize) -> Vec<(Option<Flea>, Human)> {
    let mut master_fleas = Vec::with_capacity(len);
    let mut masters = Vec::with_capacity(len);
    for i in 0 .. len {
        let master_flea = generate_fleas(1).first().cloned();
        master_fleas.push(master_flea);

        let uuid = Uuid::new_v4().to_string();
        masters.push((master_fleas[i].clone(), Human {
            passport_number: uuid.clone(),
            name: uuid.clone(),
            only_flea: master_fleas[i].clone(),
            gender: Gender::Male,
        }));
    }
    masters
}
#[test]
fn multiple_entities_test() {
    let dog_cnt = 3;
    let masters = generate_masters(1);
    let (_, master) = masters.first().unwrap();
    let dogs = generate_dogs(dog_cnt, master);
    let _gen_doc = dogs.to_document(
        "https://example.com/api",
        &Default::default(),
        "https://example.com/api".parse().unwrap(),
        Default::default(),
        Default::default(),
    );
}

#[test]
fn general_test() {
    let master_flea = Flea { id: "1".to_string(), name: "master_flea".to_string() };

    let master = Human {
        passport_number: "number".to_string(),
        name: "master_name".to_string(),
        only_flea: Some(master_flea),
        gender: Gender::Male,
    };

    let master_res = Resource {
        id: ResourceIdentifier::new("humans", "number"),
        attributes: HashMap::from_iter(vec![(
            "name".to_string(),
            serde_json::Value::String("master_name".into()),
        )])
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
        id: ResourceIdentifier::new("fleas", "a"),
        attributes: HashMap::from_iter(vec![(
            "name".to_string(),
            serde_json::Value::String("dog_flea_a".into()),
        )])
        .into(),
        links: HashMap::from_iter(vec![(
            "self".into(),
            "https://example.com/api/fleas/a".parse::<Link>().unwrap(),
        )]),
        ..Default::default()
    };

    let dog_flea_b = Flea { id: "b".to_string(), name: "dog_flea_b".to_string() };

    let dog_flea_b_res = Resource {
        id: ResourceIdentifier::new("fleas", "b"),
        attributes: HashMap::from_iter(vec![(
            "name".to_string(),
            serde_json::Value::String("dog_flea_b".into()),
        )])
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
        master: &master,
        best_one: None,
    };

    let dog_res = Resource {
        id: ResourceIdentifier::new("dogs", "1"),
        attributes: HashMap::from_iter(vec![(
            "name".to_string(),
            serde_json::Value::String("dog_name".into()),
        )])
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

    let mut document = Document::single_resource(
        dog_res,
        HashMap::from_iter(vec![
            (master_res.id.clone(), master_res),
            (dog_flea_a_res.id.clone(), dog_flea_a_res),
            (dog_flea_b_res.id.clone(), dog_flea_b_res),
        ]),
    );
    document.extend_links(HashMap::from_iter(vec![Link::slf(
        "https://example.com",
        "/api".parse::<http::Uri>().unwrap(),
    )]));

    let gen_doc: Document = dog
        .to_document(
            "https://example.com/api",
            &Query {
                fields: HashMap::from_iter(vec![(
                    "humans".into(),
                    HashSet::from_iter(vec!["name".into(), "only_flea".into()]),
                )]),
                ..Default::default()
            },
            "https://example.com/api".parse().unwrap(),
            Default::default(),
            Default::default(),
        )
        .unwrap();
    assert_eq!(document.links, gen_doc.links);

    if let (
        DocumentItem::PrimaryData(Some((PrimaryDataItem::Single(doc_res), doc_inc))),
        DocumentItem::PrimaryData(Some((PrimaryDataItem::Single(gen_res), gen_inc))),
    ) = (document.item, gen_doc.item)
    {
        assert_eq!(doc_res, gen_res);
        for (doc_key, doc_val) in doc_inc {
            if let Some(gen_val) = gen_inc.get(&doc_key) {
                assert_eq!(&doc_val, gen_val);
            } else {
                unreachable!("doc_key [{:?}] cannot be found in gen_inc", doc_key);
            }
        }
    }
}
