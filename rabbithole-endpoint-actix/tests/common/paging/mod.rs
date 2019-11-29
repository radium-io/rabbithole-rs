use crate::common::resp_to_doc;
use crate::model::dog::generate_dogs;
use actix_web::test::block_on;
use rabbithole::model::resource::{AttributeField, Resource};
use rabbithole::operation::ResourceDataWrapper;
use rabbithole::RbhResult;
use std::str::FromStr;

#[macro_export]
macro_rules! check_link {
    ($app: ident, $doc: ident, $name:ident, $data:expr) => {{
        if let Some($name) = $doc.links.get(stringify!($name)) {
            let $name = http::Uri::from($name).path_and_query().unwrap().to_string();
            let resp = crate::send_request!($app, get, $name);
            let doc = crate::common::resp_to_doc(resp).await;
            let (resources, _) = doc.into_multiple().unwrap();
            crate::common::paging::check_names(&resources, $data);
        } else {
            unreachable!("`{}` link is needed", stringify!($name));
        }
    }};
}

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

pub mod cursor_test;
pub mod offset_test;
