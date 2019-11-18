extern crate lazy_static;
extern crate rabbithole_derive as rbh_derive;

use actix_web::App;
use actix_web::{middleware, web};
use actix_web::{HttpResponse, HttpServer};
use async_trait::async_trait;

use rabbithole::operation::*;

use rabbithole_endpoint_actix::settings::ActixSettingsModel;
use rabbithole_endpoint_actix::ActixSettings;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
pub struct DogService(HashMap<String, Dog>);

impl Operation for DogService {
    type Item = Dog;
}

#[async_trait]
impl Fetching for DogService {}
#[async_trait]
impl Creating for DogService {}
#[async_trait]
impl Updating for DogService {}
#[async_trait]
impl Deleting for DogService {}

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
#[entity(type = "dogs")]
#[entity(service(DogService))]
#[entity(backend(actix))]
pub struct Dog {
    #[entity(id)]
    pub id: Uuid,
    pub name: String,
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("config/actix.config.example.toml")).unwrap();
    let settings: ActixSettingsModel = settings.try_into().unwrap();
    let settings_port = settings.port;

    HttpServer::new(move || {
        App::new()
            .register_data(web::Data::new(Mutex::new(DogService::default())))
            .data::<ActixSettings<DogService>>(settings.clone().try_into().unwrap())
            .wrap(middleware::Logger::new(r#"%a "%r" %s %b "%{Referer}i" "%{Content-Type}i" %T"#))
            .service(web::scope(&settings.path).service(DogService::actix_service()))
            .default_service(web::to(HttpResponse::NotFound))
    })
    .bind(format!("[::]:{}", settings_port))?
    .run()
}
