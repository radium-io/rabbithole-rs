#[macro_use]
extern crate lazy_static;

pub mod common;

use common::Dog;

use rabbithole::entity::SingleEntity;
use rabbithole::query::page::{CursorBasedData, OffsetBasedData, PageBasedData, PageQuery};

lazy_static! {
    pub static ref DOGS: Vec<Dog> = vec![
        Dog { id: "a".into(), name: "1".into(), age: 3 },
        Dog { id: "b".into(), name: "2".into(), age: 2 },
        Dog { id: "c".into(), name: "2".into(), age: 1 },
    ];
}

#[test]
fn offset_page_test() {
    let dogs = DOGS.clone();
    let page = PageQuery::OffsetBased(OffsetBasedData { offset: 0, limit: 2 });
    let slice = page.page(&dogs);
    assert_eq!(slice.len(), 2);
    assert_eq!(slice[0].id(), "a");
    assert_eq!(slice[1].id(), "b");
}

#[test]
fn overflow_offset_page_test() {
    let dogs = DOGS.clone();
    let page = PageQuery::OffsetBased(OffsetBasedData { offset: 0, limit: 100 });
    let slice = page.page(&dogs);
    assert_eq!(slice.len(), 3);
    assert_eq!(slice[0].id(), "a");
    assert_eq!(slice[1].id(), "b");
    assert_eq!(slice[2].id(), "c");
}

#[test]
fn larger_than_max_offset_page_test() {
    let dogs = DOGS.clone();
    let page = PageQuery::OffsetBased(OffsetBasedData { offset: 90, limit: 100 });
    let slice = page.page(&dogs);
    assert_eq!(slice.len(), 0);
}

#[test]
fn page_based_page_test() {
    let dogs = DOGS.clone();
    let page = PageQuery::PageBased(PageBasedData { number: 0, size: 2 });
    let slice = page.page(&dogs);
    assert_eq!(slice.len(), 2);
    assert_eq!(slice[0].id(), "a");
    assert_eq!(slice[1].id(), "b");

    let page = PageQuery::PageBased(PageBasedData { number: 1, size: 2 });
    let slice = page.page(&dogs);
    assert_eq!(slice.len(), 1);
    assert_eq!(slice[0].id(), "c");
}

#[test]
fn overflow_page_based_page_test() {
    let dogs = DOGS.clone();
    let page = PageQuery::PageBased(PageBasedData { number: 0, size: 100 });
    let slice = page.page(&dogs);
    assert_eq!(slice.len(), 3);
    assert_eq!(slice[0].id(), "a");
    assert_eq!(slice[1].id(), "b");
    assert_eq!(slice[2].id(), "c");

    let page = PageQuery::PageBased(PageBasedData { number: 0, size: 0 });
    let slice = page.page(&dogs);
    assert_eq!(slice.len(), 0);

    let page = PageQuery::PageBased(PageBasedData { number: 100, size: 0 });
    let slice = page.page(&dogs);
    assert_eq!(slice.len(), 0);

    let page = PageQuery::PageBased(PageBasedData { number: 100, size: 100 });
    let slice = page.page(&dogs);
    assert_eq!(slice.len(), 0);
}

#[test]
fn cursor_based_test() {
    let dogs = DOGS.clone();
    let page = PageQuery::CursorBased(CursorBasedData {
        target_id: "a".to_string(),
        is_look_after: true,
        limit: 1,
    });
    let slice = page.page(&dogs);
    assert_eq!(slice.len(), 1);
    assert_eq!(slice[0].id(), "b");

    let page = PageQuery::CursorBased(CursorBasedData {
        target_id: "a".to_string(),
        is_look_after: false,
        limit: 1,
    });
    let slice = page.page(&dogs);
    assert_eq!(slice.len(), 1);
    assert_eq!(slice[0].id(), "a");

    let page = PageQuery::CursorBased(CursorBasedData {
        target_id: "b".to_string(),
        is_look_after: true,
        limit: 2,
    });
    let slice = page.page(&dogs);
    assert_eq!(slice.len(), 1);
    assert_eq!(slice[0].id(), "c");

    let page = PageQuery::CursorBased(CursorBasedData {
        target_id: "b".to_string(),
        is_look_after: false,
        limit: 2,
    });
    let slice = page.page(&dogs);
    assert_eq!(slice.len(), 2);
    assert_eq!(slice[0].id(), "a");
    assert_eq!(slice[1].id(), "b");

    let page = PageQuery::CursorBased(CursorBasedData {
        target_id: "c".to_string(),
        is_look_after: false,
        limit: 100,
    });
    let slice = page.page(&dogs);
    assert_eq!(slice.len(), 3);
    assert_eq!(slice[0].id(), "a");
    assert_eq!(slice[1].id(), "b");
    assert_eq!(slice[2].id(), "c");

    let page = PageQuery::CursorBased(CursorBasedData {
        target_id: "c".to_string(),
        is_look_after: true,
        limit: 100,
    });
    let slice = page.page(&dogs);
    assert_eq!(slice.len(), 0);
}
