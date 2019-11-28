use crate::{init_app, send_request};
use actix_http_test::block_on;

use crate::model::dog::generate_dogs;

use rabbithole::model::resource::{AttributeField, Resource};

use crate::common::resp_to_doc;

use rabbithole::operation::ResourceDataWrapper;
use rabbithole::query::page::Cursor;
use rabbithole::RbhResult;
use std::str::FromStr;

fn get_names(resources: &[Resource]) -> Vec<String> {
    let names: RbhResult<Vec<AttributeField>> =
        resources.iter().map(|r| r.attributes.get_field("name").map(Clone::clone)).collect();
    names.unwrap().iter().map(|a| a.0.as_str().unwrap().to_string()).collect()
}

fn check_names(resources: &[Resource], names: &[usize]) {
    assert_eq!(resources.len(), names.len());

    for r in resources {
        let name =
            usize::from_str(r.attributes.get_field("name").unwrap().0.as_str().unwrap()).unwrap();
        assert!(names.contains(&name));
    }
}

#[test]
fn paging_default_test() {
    block_on(async {
        // Prepare data
        let (host, path, app) = init_app!(DefaultPage);
        let dogs = generate_dogs(7);
        let dog_resources = ResourceDataWrapper::from_entities(&dogs, &host);
        for dog in &dog_resources {
            let resp = send_request!(app, post, dog, "{}/dogs", path);
            assert!(resp.status().is_success());
        }

        let resp = send_request!(app, get, "{}/dogs", path);
        let doc = resp_to_doc(resp).await;
        assert_eq!(doc.links.len(), 1);
        assert!(doc.links.contains_key("self"));

        let (resources, _) = doc.into_multiple().unwrap();
        assert_eq!(resources.len(), 7);
        let names = get_names(&resources);
        for i in 0 .. 7 {
            assert!(names.contains(&i.to_string()));
        }
    });
}

#[test]
fn paging_cursor_range_test() {
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

        if let Some(prev) = doc.links.get("prev") {
            let prev = http::Uri::from(prev).path_and_query().unwrap().to_string();
            let resp = send_request!(app, get, prev);
            let doc = resp_to_doc(resp).await;
            let (resources, _) = doc.into_multiple().unwrap();
            check_names(&resources, &[0, 1]);
        } else {
            unreachable!("`prev` link is needed");
        }

        if let Some(next) = doc.links.get("next") {
            let next = http::Uri::from(next).path_and_query().unwrap().to_string();
            let resp = send_request!(app, get, next);
            let doc = resp_to_doc(resp).await;
            let (resources, _) = doc.into_multiple().unwrap();
            check_names(&resources, &[4, 5, 6]);
        } else {
            unreachable!("`next` link is needed");
        }

        let (resources, _) = doc.into_multiple().unwrap();
        check_names(&resources, (2usize ..= 3).collect::<Vec<usize>>().as_slice());
    });
}
