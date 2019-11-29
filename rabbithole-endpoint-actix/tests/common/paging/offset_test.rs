use crate::{init_app, send_request};
use actix_http_test::block_on;

use crate::common::paging::check_names;
use crate::common::resp_to_doc;
use crate::model::dog::generate_dogs;
use rabbithole::operation::ResourceDataWrapper;

#[test]
fn empty_test() {
    block_on(async {
        // Prepare data
        let (_, path, app) = init_app!(OffsetBased);
        let resp = send_request!(app, get, "{}/dogs?sort=name&page[limit]=10", path);
        let doc = resp_to_doc(resp).await;
        assert_eq!(doc.links.len(), 1);
        let (resources, _) = doc.into_multiple().unwrap();
        assert!(resources.is_empty());
    });
}

#[test]
fn test() {
    block_on(async {
        // Prepare data
        let (host, path, app) = init_app!(OffsetBased);

        let dogs = generate_dogs(7);
        let dog_resources = ResourceDataWrapper::from_entities(&dogs, &host);
        for dog in &dog_resources {
            let resp = send_request!(app, post, dog, "{}/dogs", path);
            assert!(resp.status().is_success());
        }

        // Zero offset test
        let resp = send_request!(app, get, "{}/dogs?sort=name&page[limit]=3", path);
        let doc = resp_to_doc(resp).await;
        assert_eq!(doc.links.len(), 4);

        check_link!(app, doc, last, &[4, 5, 6]);
        check_link!(app, doc, next, &[3, 4, 5]);
        check_link!(app, doc, first, &[0, 1, 2]);

        let (resources, _) = doc.into_multiple().unwrap();
        check_names(&resources, &[0, 1, 2]);
    });
}
