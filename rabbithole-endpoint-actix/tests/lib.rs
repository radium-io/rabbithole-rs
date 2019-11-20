#[macro_use]
extern crate lazy_static;

pub mod common;
mod model;
mod service;
pub mod v1_0;
pub mod v1_1;

#[macro_export]
macro_rules! init_app {
    ($major:expr, $minor:expr) => {{
        use std::convert::TryInto;
        let mut settings = config::Config::default();
        let version = format!("v{}_{}", $major, $minor);
        settings
            .merge(config::File::with_name(&format!("config/actix.config.test.{}.toml", version)))
            .unwrap();
        let settings: rabbithole_endpoint_actix::settings::ActixSettingsModel =
            settings.try_into().unwrap();
        let dog_service = crate::service::dog::DogService::new();
        let human_service = crate::service::human::HumanService::new(dog_service.clone());
        let path_prefix = format!("http://{}:{}{}", &settings.host, &settings.port, &settings.path);

        (
            path_prefix,
            settings.path.clone(),
            test::init_service(
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
            ),
        )
    }};
}

#[macro_export]
macro_rules! send_request {
    ($app:ident, $method:ident, $data:ident, $uri:expr, $($param:ident),*) => {{
        let req = actix_web::test::TestRequest::$method()
            .uri(&format!($uri, $($param),*))
            .header(actix_web::http::header::CONTENT_TYPE, rabbithole::JSON_API_HEADER)
            .header(actix_web::http::header::ACCEPT, rabbithole::JSON_API_HEADER)
            .set_payload(serde_json::to_string(&$data).unwrap())
            .to_request();
        let future = actix_web::test::run_on(|| $app.call(req));
        actix_web::test::block_on(future).unwrap()
    }};
    ($app:ident, $method:ident, $uri:expr, $($param:ident),*) => {{
        let req = actix_web::test::TestRequest::$method()
            .uri(&format!($uri, $($param),*))
            .header(actix_web::http::header::CONTENT_TYPE, rabbithole::JSON_API_HEADER)
            .header(actix_web::http::header::ACCEPT, rabbithole::JSON_API_HEADER)
            .to_request();
        let future = actix_web::test::run_on(|| $app.call(req));
        actix_web::test::block_on(future).unwrap()
    }}
}
