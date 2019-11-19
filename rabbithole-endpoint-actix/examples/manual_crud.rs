#[macro_use]
extern crate lazy_static;

pub mod model;
pub mod service;

use actix_web::App;
use actix_web::{middleware, web};
use actix_web::{HttpResponse, HttpServer};

use rabbithole_endpoint_actix::settings::ActixSettingsModel;
use rabbithole_endpoint_actix::ActixSettings;

use crate::service::dog::DogService;
use crate::service::human::HumanService;
use std::convert::TryInto;

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("config/actix.config.example.toml")).unwrap();
    let settings: ActixSettingsModel = settings.try_into().unwrap();
    let settings_port = settings.port;

    HttpServer::new(move || {
        let dog_service = DogService::new();
        let human_service = HumanService::new(dog_service.clone());
        App::new()
            .data(dog_service)
            .data(human_service)
            .data::<ActixSettings>(settings.clone().try_into().unwrap())
            .wrap(middleware::Logger::new(r#"%a "%r" %s %b "%{Referer}i" "%{Content-Type}i" %T"#))
            .service(
                web::scope(&settings.path)
                    .service(DogService::actix_service())
                    .service(HumanService::actix_service()),
            )
            .default_service(web::to(HttpResponse::NotFound))
    })
    .bind(format!("[::]:{}", settings_port))?
    .run()
}
