#[macro_use]
extern crate lazy_static;

pub mod common;

use common::Dog;

use rabbithole::query::filter::FilterData;
use rabbithole::query::filter::RsqlFilterData;
use std::collections::HashMap;
use std::iter::FromIterator;

lazy_static! {
    pub static ref DOGS: Vec<Dog> = vec![
        Dog { id: "a".into(), name: "123".into(), age: 3 },
        Dog { id: "b".into(), name: "124".into(), age: 2 },
        Dog { id: "c".into(), name: "321".into(), age: 1 },
    ];
}

#[test]
fn rsql_test() {
    let rsql_data = RsqlFilterData::new(&HashMap::from_iter(vec![(
        "dogs".into(),
        "name==123".into(),
    )]))
    .unwrap();
    assert_eq!(rsql_data.filter(DOGS.clone()).unwrap().len(), 1);

    let rsql_data = RsqlFilterData::new(&HashMap::from_iter(vec![(
        "dogs".into(),
        "name!=123".into(),
    )]))
    .unwrap();
    assert_eq!(rsql_data.filter(DOGS.clone()).unwrap().len(), 2);

    let rsql_data = RsqlFilterData::new(&HashMap::from_iter(vec![(
        "dogs".into(),
        "name==12*".into(),
    )]))
    .unwrap();
    assert_eq!(rsql_data.filter(DOGS.clone()).unwrap().len(), 2);
}
