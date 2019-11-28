#[macro_use]
extern crate lazy_static;

use actix_web::web;
use std::convert::TryInto;

pub mod common;
mod model;
mod service;
pub mod v1_0;
pub mod v1_1;

fn app(settings: config::Config) -> (String, String, actix_http_test::TestServerRuntime) {
    let settings: rabbithole_endpoint_actix::settings::ActixSettingsModel =
        settings.try_into().unwrap();
    let dog_service = crate::service::dog::DogService::new();
    let human_service = crate::service::human::HumanService::new(dog_service.clone());
    let path_prefix = format!("http://{}:{}{}", &settings.host, &settings.port, &settings.path);

    (
        path_prefix,
        settings.path.clone(),
        actix_http_test::TestServer::start(move || {
            actix_http::HttpService::new(
                actix_web::App::new()
                    .data(dog_service.clone())
                    .data(human_service.clone())
                    .data::<rabbithole_endpoint_actix::ActixSettings>(
                        settings.clone().try_into().unwrap(),
                    )
                    .service(
                        web::scope(&settings.path)
                            .service(crate::service::dog::DogService::actix_service())
                            .service(crate::service::human::HumanService::actix_service()),
                    )
                    .default_service(web::to(actix_web::HttpResponse::NotFound)),
            )
        }),
    )
}

#[macro_export]
macro_rules! init_app {
    (PageBased) => {{
        init_app!("config/actix.config.test.page_based.toml".to_string())
    }};
    (CursorBased) => {{
        init_app!("config/actix.config.test.cursor_based.toml".to_string())
    }};
    (OffsetBased) => {{
        init_app!("config/actix.config.test.offset_based.toml".to_string())
    }};
    (DefaultPage) => {{
        init_app!("config/actix.config.test.default_page.toml".to_string())
    }};
    ($file_name:expr) => {{
        let mut settings = config::Config::default();
        settings.merge(config::File::with_name(&$file_name)).unwrap();
        crate::app(settings)
    }};
    ($major:expr, $minor:expr) => {{
        init_app!(format!("config/actix.config.test.v{}_{}.toml", $major, $minor))
    }};
}

#[macro_export]
macro_rules! send_request {
    ($app:ident, $method:ident, $data:ident, $uri:expr, $($param:ident),*) => {{
        $app.$method(&format!($uri, $($param),*))
            .header(actix_web::http::header::CONTENT_TYPE, rabbithole::JSON_API_HEADER)
            .header(actix_web::http::header::ACCEPT, rabbithole::JSON_API_HEADER)
            .send_json(&$data).await.unwrap()
    }};
    ($app:ident, $method:ident, $uri:expr, $($param:ident),*) => {{
        let req = $app.$method(&format!($uri, $($param),*))
            .header(actix_web::http::header::CONTENT_TYPE, rabbithole::JSON_API_HEADER)
            .header(actix_web::http::header::ACCEPT, rabbithole::JSON_API_HEADER);
        req.send().await.unwrap()
    }}
}
