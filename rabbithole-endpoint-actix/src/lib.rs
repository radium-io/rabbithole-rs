pub mod settings;

use actix_web::http::{header, HeaderMap, StatusCode};
use actix_web::web;
use actix_web::{HttpRequest, HttpResponse};
use rabbithole::entity::SingleEntity;

use crate::settings::{ActixSettingsModel, JsonApiSettings};
use actix_web::dev::HttpResponseBuilder;

use rabbithole::model::error;
use rabbithole::model::version::JsonApiVersion;
use rabbithole::operation::{
    Creating, Deleting, Fetching, IdentifierDataWrapper, ResourceDataWrapper,
};
use rabbithole::rule::RuleDispatcher;
use rabbithole::JSON_API_HEADER;
use serde::export::TryFrom;

use futures::lock::Mutex;
use rabbithole::query::Query;

use std::sync::Arc;

fn error_to_response(err: error::Error) -> HttpResponse {
    new_json_api_resp(
        err.status.as_deref().and_then(|s| s.parse().ok()).unwrap_or(StatusCode::BAD_REQUEST),
    )
    .json(err)
}

#[derive(Debug, Clone)]
pub struct ActixSettings {
    pub path: String,
    pub uri: url::Url,
    pub jsonapi: JsonApiSettings,
}

impl TryFrom<ActixSettingsModel> for ActixSettings {
    type Error = url::ParseError;

    fn try_from(value: ActixSettingsModel) -> Result<Self, Self::Error> {
        let ActixSettingsModel { host, port, path, jsonapi } = value;
        let uri = format!("http://{}:{}", host, port).parse::<url::Url>().unwrap();
        let uri = uri.join(&path).unwrap();
        Ok(Self { path, uri, jsonapi })
    }
}

macro_rules! to_response {
    (Resource: $this:ident, $item:ident) => {{
        let resource = $item.to_resource(&$this.uri.to_string(), &Default::default());
        match resource {
            Some(resource) => Ok(actix_web::HttpResponse::Ok()
                .json(rabbithole::operation::ResourceDataWrapper { data: resource })),
            None => Ok(actix_web::HttpResponse::NoContent().finish()),
        }
    }};
    (Relationship: $this:ident, $item:ident) => {{
        let (field_name, item) = $item;
        let resource = item.to_resource(&$this.uri.to_string(), &Default::default());
        match resource {
            Some(resource) => Ok(actix_web::HttpResponse::Ok().json(
                rabbithole::operation::IdentifierDataWrapper {
                    data: resource.relationships.get(&field_name).cloned().unwrap().data,
                },
            )),
            None => Ok(actix_web::HttpResponse::NoContent().finish()),
        }
    }};
}

macro_rules! single_step_operation {
    ($return_ty:ident:  $fn_name:ident, $mark:ident, $( $param:ident => $ty:ty ),+) => {
        pub async fn $fn_name<T>(this: web::Data<Self>, service: actix_web::web::Data<std::sync::Arc<futures::lock::Mutex<T>>>, req: actix_web::HttpRequest, $($param: $ty),+)
          -> Result<actix_web::HttpResponse, actix_web::Error>
            where
                T: 'static + rabbithole::operation::$mark + Send + Sync,
                T::Item: rabbithole::entity::SingleEntity + Send + Sync,
          {
            if let Err(err_resp) = check_header(&this.jsonapi.version, &req.headers()) {
                return Ok(err_resp);
            }

                match service.lock().await.$fn_name($(&$param.into_inner()),+).await {
                    Ok(item) => {
                        to_response!($return_ty: this, item)
                    },
                    Err(err) => Ok(error_to_response(err)),
                }
        }
    };
}

impl ActixSettings {
    single_step_operation!(Resource: update_resource, Updating, params => web::Path<String>, body => web::Json<ResourceDataWrapper>);

    single_step_operation!(Relationship: replace_relationship, Updating, params => web::Path<(String, String)>, body => web::Json<IdentifierDataWrapper>);

    single_step_operation!(Relationship: add_relationship, Updating, params => web::Path<(String, String)>, body => web::Json<IdentifierDataWrapper>);

    single_step_operation!(Relationship: remove_relationship, Updating, params => web::Path<(String, String)>, body => web::Json<IdentifierDataWrapper>);
}

impl ActixSettings {
    pub async fn delete_resource<T>(
        this: web::Data<Self>, service: web::Data<Arc<Mutex<T>>>, params: web::Path<String>,
        req: actix_web::HttpRequest,
    ) -> Result<HttpResponse, actix_web::Error>
    where
        T: 'static + Deleting + Send + Sync,
        T::Item: rabbithole::entity::SingleEntity + Send + Sync,
    {
        if let Err(err_resp) = check_header(&this.jsonapi.version, &req.headers()) {
            return Ok(err_resp);
        }

        match service.lock().await.delete_resource(&params.into_inner()).await {
            Ok(()) => Ok(actix_web::HttpResponse::Ok().finish()),
            Err(err) => Ok(error_to_response(err)),
        }
    }
}

impl ActixSettings {
    pub async fn create<T>(
        this: web::Data<Self>, service: web::Data<Arc<Mutex<T>>>, req: actix_web::HttpRequest,
        body: web::Json<ResourceDataWrapper>,
    ) -> Result<HttpResponse, actix_web::Error>
    where
        T: 'static + Creating + Send + Sync,
        T::Item: rabbithole::entity::SingleEntity + Send + Sync,
    {
        if let Err(err_resp) = check_header(&this.jsonapi.version, &req.headers()) {
            return Ok(err_resp);
        }

        match service.lock().await.create(&body.into_inner()).await {
            Ok(item) => {
                let resource =
                    item.to_resource(&this.uri.to_string(), &Default::default()).unwrap();
                Ok(actix_web::HttpResponse::Ok().json(ResourceDataWrapper { data: resource }))
            },
            Err(err) => Ok(error_to_response(err)),
        }
    }
}

impl ActixSettings {
    pub async fn fetch_collection<T>(
        this: web::Data<Self>, service: web::Data<Arc<Mutex<T>>>, req: HttpRequest,
    ) -> Result<HttpResponse, actix_web::Error>
    where
        T: 'static + Fetching + Send + Sync,
        T::Item: rabbithole::entity::SingleEntity + Send + Sync,
    {
        if let Err(err_resp) = check_header(&this.jsonapi.version, &req.headers()) {
            return Ok(err_resp);
        }
        match Query::from_uri(req.uri()) {
            Ok(query) => {
                let vec_res = service.lock().await.fetch_collection(&query).await;
                match vec_res {
                    Ok(vec) => {
                        match T::vec_to_document(
                            &vec,
                            &this.uri.to_string(),
                            &query,
                            &req.uri().into(),
                        )
                        .await
                        {
                            Ok(doc) => Ok(HttpResponse::Ok().json(doc)),
                            Err(err) => Ok(error_to_response(err)),
                        }
                    },
                    Err(err) => Ok(error_to_response(err)),
                }
            },
            Err(err) => Ok(error_to_response(err)),
        }
    }

    pub async fn fetch_single<T>(
        this: web::Data<Self>, service: web::Data<Arc<Mutex<T>>>, param: web::Path<String>,
        req: HttpRequest,
    ) -> Result<HttpResponse, actix_web::Error>
    where
        T: 'static + Fetching + Send + Sync,
        T::Item: rabbithole::entity::SingleEntity + Send + Sync,
    {
        if let Err(err_resp) = check_header(&this.jsonapi.version, &req.headers()) {
            return Ok(err_resp);
        }
        match Query::from_uri(req.uri()) {
            Ok(query) => {
                match service.lock().await.fetch_single(&param.into_inner(), &query).await {
                    Ok(item) => {
                        match item.to_document_automatically(
                            &this.uri.to_string(),
                            &query,
                            &req.uri().into(),
                        ) {
                            Ok(doc) => Ok(new_json_api_resp(StatusCode::OK).json(doc)),
                            Err(err) => Ok(error_to_response(err)),
                        }
                    },
                    Err(err) => Ok(error_to_response(err)),
                }
            },
            Err(err) => Ok(error_to_response(err)),
        }
    }

    pub async fn fetch_relationship<T>(
        this: web::Data<Self>, service: web::Data<Arc<Mutex<T>>>,
        param: web::Path<(String, String)>, req: HttpRequest,
    ) -> Result<HttpResponse, actix_web::Error>
    where
        T: 'static + Fetching + Send + Sync,
        T::Item: rabbithole::entity::SingleEntity + Send + Sync,
    {
        if let Err(err_resp) = check_header(&this.jsonapi.version, &req.headers()) {
            return Ok(err_resp);
        }
        match Query::from_uri(req.uri()) {
            Ok(query) => {
                let (id, related_field) = param.into_inner();
                match service
                    .lock()
                    .await
                    .fetch_relationship(
                        &id,
                        &related_field,
                        &this.uri.to_string(),
                        &query,
                        &req.uri().into(),
                    )
                    .await
                {
                    Ok(item) => Ok(new_json_api_resp(StatusCode::OK).json(item)),
                    Err(err) => Ok(error_to_response(err)),
                }
            },
            Err(err) => Ok(error_to_response(err)),
        }
    }

    pub async fn fetch_related<T>(
        this: web::Data<Self>, service: web::Data<Arc<Mutex<T>>>,
        param: web::Path<(String, String)>, req: HttpRequest,
    ) -> Result<HttpResponse, actix_web::Error>
    where
        T: 'static + Fetching + Send + Sync,
        T::Item: rabbithole::entity::SingleEntity + Send + Sync,
    {
        if let Err(err_resp) = check_header(&this.jsonapi.version, &req.headers()) {
            return Ok(err_resp);
        }

        match Query::from_uri(req.uri()) {
            Ok(query) => {
                let (id, related_field) = param.into_inner();
                match service
                    .lock()
                    .await
                    .fetch_related(
                        &id,
                        &related_field,
                        &this.uri.to_string(),
                        &query,
                        &req.uri().into(),
                    )
                    .await
                {
                    Ok(item) => Ok(new_json_api_resp(StatusCode::OK).json(item)),
                    Err(err) => Ok(error_to_response(err)),
                }
            },
            Err(err) => Ok(error_to_response(err)),
        }
    }
}

// TODO: If this check should be put into the main logic rather than web-framework specific?
fn check_header(api_version: &JsonApiVersion, headers: &HeaderMap) -> Result<(), HttpResponse> {
    let content_type = headers.get(header::CONTENT_TYPE).map(|r| r.to_str().unwrap().to_string());
    let accept = headers.get(header::ACCEPT).map(|r| r.to_str().unwrap().to_string());
    RuleDispatcher::ContentTypeMustBeJsonApi(api_version, &content_type)
        .map_err(error_to_response)?;
    RuleDispatcher::AcceptHeaderShouldBeJsonApi(api_version, &accept).map_err(error_to_response)?;

    Ok(())
}

fn new_json_api_resp(status_code: StatusCode) -> HttpResponseBuilder {
    let mut resp = HttpResponse::build(status_code);
    resp.set_header(header::CONTENT_TYPE, JSON_API_HEADER);
    resp
}
