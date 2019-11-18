extern crate lazy_static;
extern crate rabbithole_derive as rbh_derive;

use actix_web::App;
use actix_web::{middleware, web};
use actix_web::{HttpResponse, HttpServer};
use async_trait::async_trait;

use rabbithole::operation::*;

use rabbithole::model::error;
use rabbithole::model::error::Error;

use rabbithole::model::resource::AttributeField;
use rabbithole::query::Query;
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

#[derive(rbh_derive::EntityDecorator, Serialize, Deserialize, Clone)]
#[entity(type = "dogs")]
#[entity(service(DogService))]
#[entity(backend(actix))]
pub struct Dog {
    #[entity(id)]
    pub id: Uuid,
    pub name: String,
}

#[async_trait]
impl Fetching for DogService {
    async fn fetch_collection(&self, _query: &Query) -> Result<Vec<Dog>, Error> {
        Ok(self.0.values().cloned().collect())
    }

    async fn fetch_single(&self, id: &str, _query: &Query) -> Result<Option<Dog>, Error> {
        Ok(self.0.get(id).map(Clone::clone))
    }
}
#[async_trait]
impl Creating for DogService {
    async fn create(&mut self, data: &ResourceDataWrapper) -> Result<Dog, Error> {
        let ResourceDataWrapper { data } = data;
        if let AttributeField(serde_json::Value::String(name)) =
            data.attributes.get_field("name")?
        {
            let dog = Dog { id: Uuid::new_v4(), name: name.clone() };
            self.0.insert(dog.id.clone().to_string(), dog.clone());
            Ok(dog)
        } else {
            Err(error::Error {
                status: Some("400".into()),
                code: Some("1".into()),
                title: Some("Wrong field type".into()),
                ..Default::default()
            })
        }
    }
}
#[async_trait]
impl Updating for DogService {}
#[async_trait]
impl Deleting for DogService {}

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
