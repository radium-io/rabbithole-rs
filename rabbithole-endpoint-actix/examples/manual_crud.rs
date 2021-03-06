use actix_web::App;
use actix_web::{middleware, web};
use actix_web::{HttpResponse, HttpServer};
use config::{Config, File};
use rabbithole_endpoint_actix::ActixSettings;

extern crate rabbithole_endpoint_actix_tests_common;
use rabbithole_endpoint_actix_tests_common::common::service::dog::DogService;
use rabbithole_endpoint_actix_tests_common::common::service::human::HumanService;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let mut settings = Config::default();
    settings
        .merge(File::with_name("config/actix.config.example.toml"))
        .unwrap();
    let actix_settings: ActixSettings = settings.try_into().unwrap();
    let service_settings = actix_settings.clone();

    let dog_service = DogService::new();
    let human_service = HumanService::new(dog_service.clone());

    use actix_web::middleware::DefaultHeaders;

    HttpServer::new(move || {
        App::new()
            .data(dog_service.clone())
            .data(human_service.clone())
            .data::<ActixSettings>(service_settings.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(
                web::scope(&service_settings.path)
                    .wrap(rabbithole_endpoint_actix::middleware::JsonApi)
                    .wrap(DefaultHeaders::new().header("Content-Type", "application/vnd.api+json"))
                    .service(DogService::actix_service())
                    .service(HumanService::actix_service()),
            )
            .default_service(web::to(HttpResponse::NotFound))
    })
    .bind(format!("[::]:{:?}", actix_settings.port))?
    .run()
    .await
}
