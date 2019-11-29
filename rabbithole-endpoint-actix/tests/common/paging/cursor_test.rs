use actix_http_test::block_on;

use crate::model::dog::generate_dogs;

use crate::common::resp_to_doc;

use rabbithole::operation::ResourceDataWrapper;
use rabbithole::query::page::Cursor;

use crate::common::paging::check_names;
use rabbithole::model::error;

#[test]
fn empty_test() {
    block_on(async {
        // Prepare data
        let (_, path, app) = init_app!(CursorBased);

        let after_cursor = Cursor { id: "1".to_string() }.to_string();
        let before_cursor = Cursor { id: "2".to_string() }.to_string();
        let resp = send_request!(
            app,
            get,
            "{}/dogs?sort=name&page[after]={}&page[before]={}&page[size]=3",
            path,
            after_cursor,
            before_cursor
        );
        let doc = resp_to_doc(resp).await;
        assert_eq!(doc.links.len(), 1);
        let (resources, _) = doc.into_multiple().unwrap();
        assert!(resources.is_empty());
    });
}

#[test]
fn range_test() {
    block_on(async {
        // Prepare data
        let (host, path, app) = init_app!(CursorBased);

        let dogs = generate_dogs(7);
        let dog_resources = ResourceDataWrapper::from_entities(&dogs, &host);
        for dog in &dog_resources {
            let resp = send_request!(app, post, dog, "{}/dogs", path);
            assert!(resp.status().is_success());
        }

        let after_cursor = Cursor { id: dogs[1].id.to_string() }.to_string();
        let before_cursor = Cursor { id: dogs[4].id.to_string() }.to_string();
        let resp = send_request!(
            app,
            get,
            "{}/dogs?sort=name&page[after]={}&page[before]={}&page[size]=3",
            path,
            after_cursor,
            before_cursor
        );
        let doc = resp_to_doc(resp).await;
        assert_eq!(doc.links.len(), 3);

        check_link!(app, doc, prev, &[0, 1]);
        check_link!(app, doc, next, &[4, 5, 6]);

        let (resources, _) = doc.into_multiple().unwrap();
        check_names(&resources, &[2, 3]);
    });
}

#[test]
fn one_side_test() {
    block_on(async {
        // Prepare data
        let (host, path, app) = init_app!(CursorBased);

        let dogs = generate_dogs(7);
        let dog_resources = ResourceDataWrapper::from_entities(&dogs, &host);
        for dog in &dog_resources {
            let resp = send_request!(app, post, dog, "{}/dogs", path);
            assert!(resp.status().is_success());
        }

        // Only after
        let after_cursor = Cursor { id: dogs[3].id.to_string() }.to_string();
        let resp = send_request!(
            app,
            get,
            "{}/dogs?sort=name&page[after]={}&page[size]=3",
            path,
            after_cursor
        );
        let doc = resp_to_doc(resp).await;
        assert_eq!(doc.links.len(), 2);

        check_link!(app, doc, prev, &[1, 2, 3]);

        let (resources, _) = doc.into_multiple().unwrap();
        check_names(&resources, &[4, 5, 6]);

        // Only before
        let before_cursor = Cursor { id: dogs[2].id.to_string() }.to_string();
        let resp = send_request!(
            app,
            get,
            "{}/dogs?sort=name&page[before]={}&page[size]=3",
            path,
            before_cursor
        );
        let doc = resp_to_doc(resp).await;
        assert_eq!(doc.links.len(), 2);

        check_link!(app, doc, next, &[2, 3, 4]);

        let (resources, _) = doc.into_multiple().unwrap();
        check_names(&resources, &[0, 1]);

        // The cursor with not-existing ID will be ignored
        let before_cursor = Cursor { id: "no exist".to_string() }.to_string();
        let resp = send_request!(
            app,
            get,
            "{}/dogs?sort=name&page[before]={}&page[size]=3",
            path,
            before_cursor
        );
        let doc = resp_to_doc(resp).await;
        assert_eq!(doc.links.len(), 2);

        check_link!(app, doc, next, &[3, 4, 5]);

        let (resources, _) = doc.into_multiple().unwrap();
        check_names(&resources, &[0, 1, 2]);

        // The cursor bad format will be ignored
        let resp = send_request!(
            app,
            get,
            "{}/dogs?sort=name&page[before]=just_a_cursor&page[size]=3",
            path
        );
        let doc = resp_to_doc(resp).await;
        let err = doc.into_errors().unwrap().first().cloned().unwrap();
        assert_eq!(err.code, error::Error::InvalidCursorContent(None).code);
    });
}
