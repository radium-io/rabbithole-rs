pub mod common;

#[macro_use]
extern crate lazy_static;

use common::Dog;
use rabbithole::entity::SingleEntity;
use rabbithole::query::sort::*;
use std::convert::TryInto;

lazy_static! {
    pub static ref DOGS: Vec<Dog> = vec![
        Dog { id: "a".into(), name: "1".into(), age: 3 },
        Dog { id: "b".into(), name: "2".into(), age: 2 },
        Dog { id: "c".into(), name: "2".into(), age: 1 },
    ];
}

#[test]
fn one_field_sorting_test() {
    let mut dogs = DOGS.clone();

    let sort_query: SortQuery = vec![("name".into(), OrderType::Asc)].try_into().unwrap();
    sort_query.sort(&mut dogs);
    assert_eq!(dogs[0].id(), "a");
    assert_eq!(dogs[1].id(), "b");
    assert_eq!(dogs[2].id(), "c");

    let sort_query: SortQuery = vec![("age".into(), OrderType::Asc)].try_into().unwrap();
    sort_query.sort(&mut dogs);
    assert_eq!(dogs[0].id(), "c");
    assert_eq!(dogs[1].id(), "b");
    assert_eq!(dogs[2].id(), "a");
}

#[test]
fn two_field_sorting_test() {
    let mut dogs = DOGS.clone();

    let sort_query: SortQuery = vec![
        ("name".into(), OrderType::Desc),
        ("age".into(), OrderType::Desc),
    ]
    .try_into()
    .unwrap();
    sort_query.sort(&mut dogs);
    assert_eq!(dogs[0].id(), "b");
    assert_eq!(dogs[1].id(), "c");
    assert_eq!(dogs[2].id(), "a");
}
