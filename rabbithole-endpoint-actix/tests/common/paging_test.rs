use crate::{init_app, send_request};
use actix_http_test::block_on;

use crate::model::dog::generate_dogs;

use rabbithole::model::resource::AttributeField;

use crate::common::resp_to_doc;

use rabbithole::operation::ResourceDataWrapper;
use rabbithole::RbhResult;

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
        eprintln!("doc: {:#?}", doc);
        assert_eq!(doc.links.len(), 1);
        assert!(doc.links.contains_key("self"));

        let (resources, _) = doc.into_multiple().unwrap();
        assert_eq!(resources.len(), 7);
        let names: RbhResult<Vec<AttributeField>> =
            resources.iter().map(|r| r.attributes.get_field("name").map(Clone::clone)).collect();
        let names: Vec<String> =
            names.unwrap().iter().map(|a| a.0.as_str().unwrap().to_string()).collect();
        for i in 0 .. 7 {
            assert!(names.contains(&i.to_string()));
        }
    });
}
