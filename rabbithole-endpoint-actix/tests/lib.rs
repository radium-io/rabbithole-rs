pub mod common;
pub mod v1_0;
pub mod v1_1;

#[macro_export]
macro_rules! fetching_init {
    () => {
        use rabbithole::entity::{Entity, SingleEntity};

        #[derive(Default)]
        pub struct DogService;

        impl rabbithole::operation::Operation for DogService {
            type Item = Dog;
        }

        #[async_trait::async_trait]
        impl rabbithole::operation::Creating for DogService {}
        #[async_trait::async_trait]
        impl rabbithole::operation::Updating for DogService {}
        #[async_trait::async_trait]
        impl rabbithole::operation::Deleting for DogService {}

        #[async_trait::async_trait]
        impl rabbithole::operation::Fetching for DogService {
            async fn fetch_collection(
                &self, _query: &rabbithole::query::Query,
            ) -> Result<Vec<Dog>, rabbithole::model::error::Error> {
                Ok(Default::default())
            }

            async fn fetch_single(
                &self, id: &str, _query: &rabbithole::query::Query,
            ) -> Result<Option<Dog>, rabbithole::model::error::Error> {
                if id == "none" {
                    Ok(None)
                } else {
                    let rand = rand::random::<usize>() % 3 + 1;
                    Ok(generate_dogs(rand).first().cloned())
                }
            }

            async fn fetch_relationship(
                &self, _: &str, related_field: &str, _: &str, _: &rabbithole::query::Query,
                _: &rabbithole::model::link::RawUri,
            ) -> Result<
                rabbithole::model::relationship::Relationship,
                rabbithole::model::error::Error,
            > {
                Err(rabbithole::model::error::Error::FieldNotExist(related_field, None))
            }

            async fn fetch_related(
                &self, _: &str, related_field: &str, _: &str, _: &rabbithole::query::Query,
                _: &rabbithole::model::link::RawUri,
            ) -> Result<serde_json::Value, rabbithole::model::error::Error> {
                Err(rabbithole::model::error::Error::FieldNotExist(related_field, None))
            }
        }

        #[derive(Default)]
        pub struct HumanService;

        impl rabbithole::operation::Operation for HumanService {
            type Item = Human;
        }

        #[async_trait::async_trait]
        impl rabbithole::operation::Creating for HumanService {}
        #[async_trait::async_trait]
        impl rabbithole::operation::Updating for HumanService {}
        #[async_trait::async_trait]
        impl rabbithole::operation::Deleting for HumanService {}

        #[async_trait::async_trait]
        impl rabbithole::operation::Fetching for HumanService {
            async fn fetch_collection(
                &self, _: &rabbithole::query::Query,
            ) -> Result<Vec<Human>, rabbithole::model::error::Error> {
                let rand = rand::random::<usize>() % 5 + 1;
                let masters = generate_masters(rand);
                Ok(masters)
            }

            async fn fetch_single(
                &self, id: &str, _query: &rabbithole::query::Query,
            ) -> Result<Option<Human>, rabbithole::model::error::Error> {
                if id == "none" {
                    Ok(None)
                } else {
                    let rand = rand::random::<usize>() % 3 + 1;
                    Ok(generate_masters(rand).first().cloned())
                }
            }

            async fn fetch_relationship(
                &self, id: &str, related_field: &str, uri: &str, _query: &rabbithole::query::Query,
                _request_path: &rabbithole::model::link::RawUri,
            ) -> Result<
                rabbithole::model::relationship::Relationship,
                rabbithole::model::error::Error,
            > {
                if related_field == "dogs" {
                    if id == "none" {
                        return Err(rabbithole::model::error::Error::ParentResourceNotExist(
                            related_field,
                            None,
                        ));
                    }

                    let rand = rand::random::<usize>() % 3 + 1;
                    let relats = generate_masters(rand).last().cloned().unwrap().relationships(uri);
                    Ok(relats.get(related_field).cloned().unwrap())
                } else {
                    Err(rabbithole::model::error::Error::FieldNotExist(related_field, None))
                }
            }

            async fn fetch_related(
                &self, id: &str, related_field: &str, uri: &str, query: &rabbithole::query::Query,
                request_path: &rabbithole::model::link::RawUri,
            ) -> Result<serde_json::Value, rabbithole::model::error::Error> {
                if id == "none" {
                    return Err(rabbithole::model::error::Error::ParentResourceNotExist(
                        related_field,
                        None,
                    ));
                }
                if related_field == "dogs" {
                    let rand = rand::random::<usize>() % 3 + 1;
                    let master = generate_masters(rand);
                    let master = master.last().unwrap();
                    serde_json::to_value(master.dogs.to_document_automatically(
                        uri,
                        query,
                        request_path,
                    )?)
                    .map_err(|err| rabbithole::model::error::Error::InvalidJson(&err, None))
                } else {
                    Err(rabbithole::model::error::Error::FieldNotExist(related_field, None))
                }
            }
        }
    };
}

#[macro_export]
macro_rules! classes_init {
    () => {
        #[derive(
            rabbithole_derive::EntityDecorator, serde::Serialize, serde::Deserialize, Clone,
        )]
        #[entity(type = "people")]
        #[entity(backend(actix))]
        #[entity(service(HumanService))]
        pub struct Human {
            #[entity(id)]
            pub id_code: uuid::Uuid,
            pub name: String,
            #[entity(to_many)]
            pub dogs: Vec<Dog>,
        }

        #[derive(
            rabbithole_derive::EntityDecorator, serde::Serialize, serde::Deserialize, Clone,
        )]
        #[entity(type = "dogs")]
        #[entity(backend(actix))]
        #[entity(service(DogService))]
        pub struct Dog {
            #[entity(id)]
            pub id: uuid::Uuid,
            pub name: String,
        }

        impl From<&[Dog]> for Human {
            fn from(dogs: &[Dog]) -> Self {
                let uuid = uuid::Uuid::new_v4();
                Self { id_code: uuid, name: uuid.to_string(), dogs: dogs.to_vec() }
            }
        }

        fn generate_dogs(len: usize) -> Vec<Dog> {
            let mut dogs = Vec::with_capacity(len);
            for _ in 0 .. len {
                let uuid = uuid::Uuid::new_v4();
                dogs.push(Dog { id: uuid, name: uuid.to_string() });
            }
            dogs
        }

        fn generate_masters(len: usize) -> Vec<Human> {
            let mut masters = Vec::with_capacity(len);
            for i in 1 ..= len {
                let uuid = uuid::Uuid::new_v4();
                let dogs = generate_dogs(i);
                masters.push(Human { id_code: uuid, name: uuid.to_string(), dogs });
            }
            masters
        }
    };
}

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

        (
            settings.path.clone(),
            test::init_service(
                actix_web::App::new()
                    .register_data(web::Data::new(futures::lock::Mutex::new(
                        HumanService::default(),
                    )))
                    .register_data(web::Data::new(futures::lock::Mutex::new(DogService::default())))
                    .data::<rabbithole_endpoint_actix::ActixSettings<HumanService>>(
                        settings.clone().try_into().unwrap(),
                    )
                    .data::<rabbithole_endpoint_actix::ActixSettings<DogService>>(
                        settings.clone().try_into().unwrap(),
                    )
                    .service(
                        web::scope(&settings.path)
                            .service(HumanService::actix_service())
                            .service(DogService::actix_service()),
                    )
                    .default_service(web::to(actix_web::HttpResponse::NotFound)),
            ),
        )
    }};
}
