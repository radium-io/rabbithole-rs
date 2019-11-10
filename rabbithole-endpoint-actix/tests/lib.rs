pub mod common;
pub mod v1_0;
pub mod v1_1;

#[macro_export]
macro_rules! fetching_init {
    () => {
        use rabbithole::entity::{Entity, SingleEntity};

        #[async_trait::async_trait]
        impl rabbithole::operation::Fetching for Dog {
            type Item = Dog;

            async fn vec_to_document(
                items: &[Self::Item], uri: &str, query: &rabbithole::model::query::Query,
                request_path: &rabbithole::model::link::RawUri,
            ) -> Result<rabbithole::model::document::Document, rabbithole::model::error::Error>
            {
                Ok(items.to_document_automatically(uri, query, request_path))
            }

            async fn fetch_collection(
                _query: &rabbithole::model::query::Query,
            ) -> Result<Vec<Self::Item>, rabbithole::model::error::Error> {
                Ok(Default::default())
            }

            async fn fetch_single(
                id: &str, _query: &rabbithole::model::query::Query,
            ) -> Result<Option<Self::Item>, rabbithole::model::error::Error> {
                if id == "none" {
                    Ok(None)
                } else {
                    let rand = rand::random::<usize>() % 3;
                    Ok(generate_dogs(rand).first().cloned())
                }
            }

            async fn fetch_relationship(
                _: &str, related_field: &str, _: &str, _: &rabbithole::model::query::Query,
                _: &rabbithole::model::link::RawUri,
            ) -> Result<
                rabbithole::model::relationship::Relationship,
                rabbithole::model::error::Error,
            > {
                Err(rabbithole::model::error::Error::RelationshipFieldNotExist(related_field, None))
            }

            async fn fetch_related(
                _: &str, related_field: &str, _: &str, _: &rabbithole::model::query::Query,
                _: &rabbithole::model::link::RawUri,
            ) -> Result<rabbithole::model::document::Document, rabbithole::model::error::Error>
            {
                Err(rabbithole::model::error::Error::RelatedFieldNotExist(related_field, None))
            }
        }

        #[async_trait::async_trait]
        impl rabbithole::operation::Fetching for Human {
            type Item = Human;

            async fn vec_to_document(
                items: &[Self::Item], uri: &str, query: &rabbithole::model::query::Query,
                request_path: &rabbithole::model::link::RawUri,
            ) -> Result<rabbithole::model::document::Document, rabbithole::model::error::Error>
            {
                Ok(items.to_document_automatically(uri, query, request_path))
            }

            async fn fetch_collection(
                _: &rabbithole::model::query::Query,
            ) -> Result<Vec<Self::Item>, rabbithole::model::error::Error> {
                let rand = rand::random::<usize>() % 5 + 1;
                let masters = generate_masters(rand);
                Ok(masters)
            }

            async fn fetch_single(
                id: &str, _query: &rabbithole::model::query::Query,
            ) -> Result<Option<Self::Item>, rabbithole::model::error::Error> {
                if id == "none" {
                    Ok(None)
                } else {
                    let rand = rand::random::<usize>() % 3 + 1;
                    Ok(generate_masters(rand).first().cloned())
                }
            }

            async fn fetch_relationship(
                id: &str, related_field: &str, uri: &str, _query: &rabbithole::model::query::Query,
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

                    let rand = rand::random::<usize>() % 3;
                    let relats = generate_masters(rand).last().cloned().unwrap().relationships(uri);
                    Ok(relats.get(related_field).cloned().unwrap())
                } else {
                    Err(rabbithole::model::error::Error::RelationshipFieldNotExist(
                        related_field,
                        None,
                    ))
                }
            }

            async fn fetch_related(
                id: &str, related_field: &str, uri: &str, query: &rabbithole::model::query::Query,
                request_path: &rabbithole::model::link::RawUri,
            ) -> Result<rabbithole::model::document::Document, rabbithole::model::error::Error>
            {
                if related_field == "dogs" {
                    if id == "none" {
                        return Err(rabbithole::model::error::Error::ParentResourceNotExist(
                            related_field,
                            None,
                        ));
                    }

                    let rand = rand::random::<usize>() % 3;
                    let master = generate_masters(rand);
                    let master = master.last().unwrap();
                    let doc = master.dogs.to_document_automatically(uri, query, request_path);
                    Ok(doc)
                } else {
                    Err(rabbithole::model::error::Error::RelatedFieldNotExist(related_field, None))
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
            for i in 0 ..= len {
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
        let _settings_port = settings.port;

        (
            settings.path.clone(),
            test::init_service(
                actix_web::App::new()
                    .data::<rabbithole_endpoint_actix::ActixSettings<Human>>(
                        settings.clone().try_into().unwrap(),
                    )
                    .data::<rabbithole_endpoint_actix::ActixSettings<Dog>>(
                        settings.clone().try_into().unwrap(),
                    )
                    .service(
                        web::scope(&settings.path)
                            .service(Human::actix_service())
                            .service(Dog::actix_service()),
                    )
                    .default_service(web::to(actix_web::HttpResponse::NotFound)),
            ),
        )
    }};
}
