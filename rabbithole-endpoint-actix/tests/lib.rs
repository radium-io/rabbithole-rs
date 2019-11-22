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
