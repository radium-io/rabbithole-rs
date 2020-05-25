#[macro_use]
extern crate lazy_static;

pub mod common;

use common::Dog;
use rabbithole::entity::SingleEntity;
use rabbithole::query::page::{Cursor, CursorBasedData, OffsetBasedData, PageBasedData, PageQuery};

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
    let page = PageQuery::OffsetBased(OffsetBasedData {
        offset: 0,
        limit: 2,
    });
    let (slice, _relat_pages) = page.page(&dogs).unwrap();
    assert_eq!(slice.len(), 2);
    assert_eq!(slice[0].id(), "a");
    assert_eq!(slice[1].id(), "b");
}

#[test]
fn overflow_offset_page_test() {
    let dogs = DOGS.clone();
    let page = PageQuery::OffsetBased(OffsetBasedData {
        offset: 0,
        limit: 100,
    });
    let (slice, _relat_pages) = page.page(&dogs).unwrap();
    assert_eq!(slice.len(), 3);
    assert_eq!(slice[0].id(), "a");
    assert_eq!(slice[1].id(), "b");
    assert_eq!(slice[2].id(), "c");
}

#[test]
fn larger_than_max_offset_page_test() {
    let dogs = DOGS.clone();
    let page = PageQuery::OffsetBased(OffsetBasedData {
        offset: 90,
        limit: 100,
    });
    let (slice, _relat_pages) = page.page(&dogs).unwrap();
    assert_eq!(slice.len(), 0);
}

#[test]
fn page_based_page_test() {
    let dogs = DOGS.clone();
    let page = PageQuery::PageBased(PageBasedData { number: 0, size: 2 });
    let (slice, _relat_pages) = page.page(&dogs).unwrap();
    assert_eq!(slice.len(), 2);
    assert_eq!(slice[0].id(), "a");
    assert_eq!(slice[1].id(), "b");

    let page = PageQuery::PageBased(PageBasedData { number: 1, size: 2 });
    let (slice, _relat_pages) = page.page(&dogs).unwrap();
    assert_eq!(slice.len(), 1);
    assert_eq!(slice[0].id(), "c");
}

#[test]
fn overflow_page_based_page_test() {
    let dogs = DOGS.clone();
    let page = PageQuery::PageBased(PageBasedData {
        number: 0,
        size: 100,
    });
    let (slice, _relat_pages) = page.page(&dogs).unwrap();
    assert_eq!(slice.len(), 3);
    assert_eq!(slice[0].id(), "a");
    assert_eq!(slice[1].id(), "b");
    assert_eq!(slice[2].id(), "c");

    let page = PageQuery::PageBased(PageBasedData { number: 0, size: 0 });
    let res = page.page(&dogs);
    assert!(res.is_err());

    let page = PageQuery::PageBased(PageBasedData {
        number: 100,
        size: 0,
    });
    let res = page.page(&dogs);
    assert!(res.is_err());

    let page = PageQuery::PageBased(PageBasedData {
        number: 100,
        size: 100,
    });
    let (slice, _relat_pages) = page.page(&dogs).unwrap();
    assert_eq!(slice.len(), 0);
}

#[test]
fn cursor_based_test() {
    let dogs = DOGS.clone();
    let page = PageQuery::CursorBased(CursorBasedData {
        after: Some(Cursor { id: "a".into() }),
        before: None,
        size: 1,
    });
    let (slice, _relat_pages) = page.page(&dogs).unwrap();
    assert_eq!(slice.len(), 1);
    assert_eq!(slice[0].id(), "b");

    let page = PageQuery::CursorBased(CursorBasedData {
        before: Some(Cursor { id: "b".into() }),
        after: None,
        size: 1,
    });
    let (slice, _relat_pages) = page.page(&dogs).unwrap();
    assert_eq!(slice.len(), 1);
    assert_eq!(slice[0].id(), "a");

    let page = PageQuery::CursorBased(CursorBasedData {
        after: Some(Cursor { id: "b".into() }),
        before: None,
        size: 2,
    });
    let (slice, _relat_pages) = page.page(&dogs).unwrap();
    assert_eq!(slice.len(), 1);
    assert_eq!(slice[0].id(), "c");

    let page = PageQuery::CursorBased(CursorBasedData {
        before: Some(Cursor { id: "c".into() }),
        after: None,
        size: 2,
    });
    let (slice, _relat_pages) = page.page(&dogs).unwrap();
    assert_eq!(slice.len(), 2);
    assert_eq!(slice[0].id(), "a");
    assert_eq!(slice[1].id(), "b");

    let page = PageQuery::CursorBased(CursorBasedData {
        before: None,
        after: None,
        size: 100,
    });
    let (slice, _relat_pages) = page.page(&dogs).unwrap();
    assert_eq!(slice.len(), 3);
    assert_eq!(slice[0].id(), "a");
    assert_eq!(slice[1].id(), "b");
    assert_eq!(slice[2].id(), "c");

    let page = PageQuery::CursorBased(CursorBasedData {
        before: Some(Cursor { id: "c".into() }),
        after: Some(Cursor { id: "a".into() }),
        size: 100,
    });
    let (slice, _relat_pages) = page.page(&dogs).unwrap();
    assert_eq!(slice.len(), 1);
    assert_eq!(slice[0].id(), "b");

    let page = PageQuery::CursorBased(CursorBasedData {
        before: Some(Cursor { id: "a".into() }),
        after: Some(Cursor { id: "c".into() }),
        size: 100,
    });
    let result = page.page(&dogs);
    assert!(result.is_err());
}
