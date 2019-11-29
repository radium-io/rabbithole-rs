use crate::common::resp_to_doc;
use crate::model::dog::generate_dogs;
use crate::{init_app, send_request};
use actix_web::test::block_on;
use rabbithole::model::resource::{AttributeField, Resource};
use rabbithole::operation::ResourceDataWrapper;
use rabbithole::RbhResult;

pub mod cursor_test;

fn get_names(resources: &[Resource]) -> Vec<String> {
    let names: RbhResult<Vec<AttributeField>> =
        resources.iter().map(|r| r.attributes.get_field("name").map(Clone::clone)).collect();
    names.unwrap().iter().map(|a| a.0.as_str().unwrap().to_string()).collect()
}

#[test]
fn default_test() {
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
