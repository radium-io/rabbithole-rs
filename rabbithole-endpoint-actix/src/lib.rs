pub mod middleware;
pub mod settings;

use crate::settings::JsonApiSettings;
use actix_web::http::StatusCode;
use actix_web::web;
use actix_web::{HttpRequest, HttpResponse};
use futures::lock::Mutex;
use rabbithole::entity::{Entity, SingleEntity};
use rabbithole::model::document::Document;
use rabbithole::model::error;
use rabbithole::operation::{
    Creating, Deleting, Fetching, IdentifierDataWrapper, OperationResultData, ResourceDataWrapper,
};
use rabbithole::query::QuerySettings;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Debug, Clone)]
pub struct ActixSettings {
    pub host: String,
    pub path: String,
    pub port: u32,
    pub jsonapi: JsonApiSettings,
    pub query: QuerySettings,
}

macro_rules! to_response {
    (Resource: $this:ident, $item:ident) => {{
        let rabbithole::operation::OperationResultData {
            data,
            additional_links,
            additional_meta,
        } = $item;
        let resource = data.to_resource(&$this.uri().to_string(), &Default::default());
        match resource {
            Some(mut resource) => {
                resource.extend_meta(additional_meta);
                resource.extend_links(additional_links);
                Ok(HttpResponse::Ok()
                    .json(rabbithole::operation::ResourceDataWrapper { data: resource }))
            },
            None => Ok(HttpResponse::NoContent().finish()),
        }
    }};
    (Relationship: $this:ident, $item:ident) => {{
        let rabbithole::operation::OperationResultData {
            data,
            additional_links,
            additional_meta,
        } = $item;
        let (field_name, item) = data;
        let resource = item.to_resource(&$this.uri().to_string(), &Default::default());
        match resource {
            Some(mut resource) => {
                resource.extend_meta(additional_meta);
                resource.extend_links(additional_links);

                Ok(
                    HttpResponse::Ok().json(rabbithole::operation::IdentifierDataWrapper {
                        data: resource
                            .relationships
                            .get(&field_name)
                            .cloned()
                            .unwrap()
                            .data,
                    }),
                )
            },
            None => Ok(HttpResponse::NoContent().finish()),
        }
    }};
}

macro_rules! single_step_operation {
    ($return_ty:ident:  $fn_name:ident, $mark:ident, $( $param:ident => $ty:ty ),+) => {
        pub async fn $fn_name<T>(this: web::Data<Self>, service: actix_web::web::Data<std::sync::Arc<futures::lock::Mutex<T>>>, req: actix_web::HttpRequest, $($param: $ty),+)
          -> actix_web::Result<HttpResponse>
            where
                T: 'static + rabbithole::operation::$mark + Send + Sync,
                T::Item: rabbithole::entity::SingleEntity + Send + Sync,
          {
            match service.lock().await.$fn_name($(&$param),+, &this.uri().to_string(), &req.uri()).await {
                Ok(item) => {
                    to_response!($return_ty: this, item)
                },
                Err(err) => ActixRabbitholeError(err).into(),
            }
        }
    };
}

impl ActixSettings {
    single_step_operation!(Resource: update_resource, Updating, params => web::Path<String>, body => web::Json<ResourceDataWrapper>);

    single_step_operation!(Relationship: replace_relationship, Updating, params => web::Path<(String, String)>, body => web::Json<IdentifierDataWrapper>);

    single_step_operation!(Relationship: add_relationship, Updating, params => web::Path<(String, String)>, body => web::Json<IdentifierDataWrapper>);

    single_step_operation!(Relationship: remove_relationship, Updating, params => web::Path<(String, String)>, body => web::Json<IdentifierDataWrapper>);

    fn uri(&self) -> url::Url {
        format!("http://{}:{}", self.host, self.port)
            .parse::<url::Url>()
            .unwrap()
            .join(&self.path)
            .unwrap()
    }

    pub async fn delete_resource<T>(
        this: web::Data<Self>,
        service: web::Data<Arc<Mutex<T>>>,
        params: web::Path<String>,
        req: actix_web::HttpRequest,
    ) -> actix_web::Result<HttpResponse>
    where
        T: 'static + Deleting + Send + Sync,
        T::Item: rabbithole::entity::SingleEntity + Send + Sync,
    {
        match service
            .lock()
            .await
            .delete_resource(&params, &this.uri().to_string(), &req.uri())
            .await
        {
            Ok(OperationResultData {
                additional_links,
                additional_meta,
                ..
            }) => {
                if additional_links.is_empty() && additional_meta.is_empty() {
                    Ok(HttpResponse::NoContent().finish())
                } else {
                    Ok(HttpResponse::Ok().json(Document::null(additional_links, additional_meta)))
                }
            },
            Err(err) => ActixRabbitholeError(err).into(),
        }
    }

    pub async fn create<T>(
        this: web::Data<Self>,
        service: web::Data<Arc<Mutex<T>>>,
        req: actix_web::HttpRequest,
        body: web::Json<ResourceDataWrapper>,
    ) -> actix_web::Result<HttpResponse>
    where
        T: 'static + Creating + Send + Sync,
        T::Item: rabbithole::entity::SingleEntity + Send + Sync,
    {
        let uri = &this.uri().to_string();

        match service
            .lock()
            .await
            .create(&body, uri, &req.uri().clone().into())
            .await
        {
            Ok(OperationResultData {
                data,
                additional_links,
                additional_meta,
            }) => {
                let mut resource = data.to_resource(uri, &Default::default()).unwrap();
                resource.extend_meta(additional_meta);
                resource.extend_links(additional_links);
                Ok(HttpResponse::Ok().json(ResourceDataWrapper { data: resource }))
            },
            Err(err) => ActixRabbitholeError(err).into(),
        }
    }

    pub async fn fetch_collection<T>(
        this: web::Data<Self>,
        service: web::Data<Arc<Mutex<T>>>,
        req: HttpRequest,
    ) -> actix_web::Result<HttpResponse>
    where
        T: 'static + Fetching + Send + Sync,
        T::Item: rabbithole::entity::SingleEntity + Send + Sync,
    {
        let uri = &this.uri().to_string();
        let path = req.uri().to_owned();

        match this.query.decode_path(&path) {
            Ok(query) => match service
                .lock()
                .await
                .fetch_collection(uri, &path, &query)
                .await
            {
                Ok(OperationResultData {
                    data,
                    additional_links,
                    additional_meta,
                }) => {
                    match data.to_document(uri, &query, path, additional_links, additional_meta) {
                        Ok(doc) => Ok(HttpResponse::Ok().json(doc)),
                        Err(err) => ActixRabbitholeError(err).into(),
                    }
                },
                Err(err) => ActixRabbitholeError(err).into(),
            },
            Err(err) => ActixRabbitholeError(err).into(),
        }
    }

    pub async fn fetch_single<T>(
        this: web::Data<Self>,
        service: web::Data<Arc<Mutex<T>>>,
        param: web::Path<String>,
        req: HttpRequest,
    ) -> actix_web::Result<HttpResponse>
    where
        T: 'static + Fetching + Send + Sync,
        T::Item: rabbithole::entity::SingleEntity + Send + Sync,
    {
        let path = req.uri().clone().into();

        match this.query.decode_path(&path) {
            Ok(query) => {
                match service
                    .lock()
                    .await
                    .fetch_single(&param, &this.uri().to_string(), &path, &query)
                    .await
                {
                    Ok(OperationResultData {
                        data,
                        additional_links,
                        additional_meta,
                    }) => {
                        match SingleEntity::to_document(
                            &data,
                            &this.uri().to_string(),
                            &query,
                            req.uri().to_owned(),
                            additional_links,
                            additional_meta,
                        ) {
                            Ok(doc) => Ok(HttpResponse::Ok().json(doc)),
                            Err(err) => ActixRabbitholeError(err).into(),
                        }
                    },
                    Err(err) => ActixRabbitholeError(err).into(),
                }
            },
            Err(err) => ActixRabbitholeError(err).into(),
        }
    }

    pub async fn fetch_relationship<T>(
        this: web::Data<Self>,
        service: web::Data<Arc<Mutex<T>>>,
        param: web::Path<(String, String)>,
        req: HttpRequest,
    ) -> actix_web::Result<HttpResponse>
    where
        T: 'static + Fetching + Send + Sync,
        T::Item: rabbithole::entity::SingleEntity + Send + Sync,
    {
        let path = req.uri().clone().into();

        match this.query.decode_path(&path) {
            Ok(query) => {
                let (id, related_field) = param.into_inner();
                match service
                    .lock()
                    .await
                    .fetch_relationship(&id, &related_field, &this.uri().to_string(), &path, &query)
                    .await
                {
                    Ok(OperationResultData {
                        mut data,
                        additional_links,
                        additional_meta,
                    }) => {
                        data.extend_links(additional_links);
                        data.extend_meta(additional_meta);
                        Ok(HttpResponse::Ok().json(data))
                    },
                    Err(err) => ActixRabbitholeError(err).into(),
                }
            },
            Err(err) => ActixRabbitholeError(err).into(),
        }
    }

    pub async fn fetch_related<T>(
        this: web::Data<Self>,
        service: web::Data<Arc<Mutex<T>>>,
        param: web::Path<(String, String)>,
        req: HttpRequest,
    ) -> actix_web::Result<HttpResponse>
    where
        T: 'static + Fetching + Send + Sync,
        T::Item: rabbithole::entity::SingleEntity + Send + Sync,
    {
        let path = req.uri().clone().into();

        match this.query.decode_path(&path) {
            Ok(query) => {
                let (id, related_field) = param.into_inner();
                service
                    .lock()
                    .await
                    .fetch_related(&id, &related_field, &this.uri().to_string(), &path, &query)
                    .await
                    .map_or_else(
                        |e| ActixRabbitholeError(e).into(),
                        |v| Ok(HttpResponse::Ok().json(v)),
                    )
            },
            Err(err) => ActixRabbitholeError(err).into(),
        }
    }
}

struct ActixRabbitholeError(error::Error);
impl From<ActixRabbitholeError> for Result<HttpResponse, actix_web::Error> {
    fn from(err: ActixRabbitholeError) -> Self {
        Ok(HttpResponse::build(
            err.0
                .status
                .as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or(StatusCode::BAD_REQUEST),
        )
        .body(serde_json::to_string(&err.0).unwrap()))
    }
}
