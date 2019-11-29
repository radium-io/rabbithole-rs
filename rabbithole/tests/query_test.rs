pub mod common;

#[macro_use]
extern crate lazy_static;

use common::Dog;
use rabbithole::entity::Entity;

use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use rabbithole::model::document::DocumentItem;

use rabbithole::query::page::{Cursor, CursorBasedData, PageQuery};
use rabbithole::query::sort::OrderType;
use rabbithole::query::Query;
use std::convert::TryInto;

lazy_static! {
    pub static ref DOGS: Vec<Dog> = vec![
        Dog { id: "a".into(), name: "1".into(), age: 3 },
        Dog { id: "b".into(), name: "2".into(), age: 2 },
        Dog { id: "c".into(), name: "2".into(), age: 1 },
    ];
}

#[test]
fn sort_and_page_test() {
    let mut dogs: Vec<Dog> = DOGS.clone();
    let query = Query {
        include: None,
        fields: Default::default(),
        sort: vec![("name".into(), OrderType::Desc), ("age".into(), OrderType::Desc)]
            .try_into()
            .unwrap(),
        page: Some(PageQuery::CursorBased(CursorBasedData {
            after: Some(Cursor { id: "b".to_string() }),
            before: None,
            size: 10,
        })),
        ..Default::default()
    };

    let uri = "sort=-name,-age&page[before]=<some-base64>&page[size]=10";
    let uri = percent_encode(uri.as_bytes(), NON_ALPHANUMERIC);
    let uri = format!("/dogs?{}", uri.to_string());

    query.sort.sort(&mut dogs);
    let (dogs, _) = query.page.as_ref().unwrap().page(&dogs).unwrap();

    let doc = dogs
        .to_document(
            "http://example.com",
            &query,
            uri.parse().unwrap(),
            Default::default(),
            Default::default(),
        )
        .unwrap();
    if let DocumentItem::PrimaryData(Some((data, _))) = doc.item {
        let data = data.data();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0].id.id, "c");
        assert_eq!(data[1].id.id, "a");
    }
}
