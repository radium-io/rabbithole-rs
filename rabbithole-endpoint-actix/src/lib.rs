pub mod settings;

use actix_web::http::{header, HeaderMap, StatusCode};
use actix_web::web;
use actix_web::{HttpRequest, HttpResponse};
use futures::{FutureExt, TryFutureExt};
use rabbithole::entity::SingleEntity;

use crate::settings::{ActixSettingsModel, JsonApiSettings};
use actix_web::dev::HttpResponseBuilder;

use rabbithole::model::error;
use rabbithole::model::version::JsonApiVersion;
use rabbithole::operation::Fetching;
use rabbithole::rule::RuleDispatcher;
use rabbithole::JSON_API_HEADER;
use serde::export::TryFrom;

use rabbithole::query::Query;
use std::marker::PhantomData;

fn error_to_response(err: error::Error) -> HttpResponse {
    new_json_api_resp(
        err.status.as_deref().and_then(|s| s.parse().ok()).unwrap_or(StatusCode::BAD_REQUEST),
    )
    .json(err)
}

#[derive(Debug, Clone)]
pub struct ActixSettings<T>
where
    T: 'static + Fetching,
{
    pub path: String,
    pub uri: url::Url,
    pub jsonapi: JsonApiSettings,
    _data: PhantomData<T>,
}

impl<T> TryFrom<ActixSettingsModel> for ActixSettings<T>
where
    T: 'static + Fetching + Send + Sync,
    T::Item: Send + Sync,
{
    type Error = url::ParseError;

    fn try_from(value: ActixSettingsModel) -> Result<Self, Self::Error> {
        let ActixSettingsModel { host, port, path, jsonapi } = value;
        let uri = format!("http://{}:{}", host, port).parse::<url::Url>().unwrap();
        let uri = uri.join(&path).unwrap();
        Ok(Self { path, uri, jsonapi, _data: PhantomData })
    }
}

impl<T> ActixSettings<T>
where
    T: 'static + Fetching + Send + Sync,
    T::Item: Send + Sync,
{
    pub fn fetch_collection(
        self, req: HttpRequest,
    ) -> impl futures01::Future<Item = HttpResponse, Error = actix_web::Error> {
        if let Err(err_resp) = check_header(&self.jsonapi.version, &req.headers()) {
            return futures::future::ok(err_resp).boxed_local().compat();
        }
        match Query::from_uri(req.uri()) {
            Ok(query) => {
                let fut = async move {
                    let vec_res = T::fetch_collection(&query).await;
                    match vec_res {
                        Ok(vec) => {
                            match T::vec_to_document(
                                &vec,
                                &self.uri.to_string(),
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
                };

                fut.boxed_local().compat()
            },
            Err(err) => futures::future::ok(error_to_response(err)).boxed_local().compat(),
        }
    }

    pub fn fetch_single(
        self, param: web::Path<String>, req: HttpRequest,
    ) -> impl futures01::Future<Item = HttpResponse, Error = actix_web::Error> {
        if let Err(err_resp) = check_header(&self.jsonapi.version, &req.headers()) {
            return futures::future::ok(err_resp).boxed_local().compat();
        }
        match Query::from_uri(req.uri()) {
            Ok(query) => {
                let fut = async move {
                    match T::fetch_single(&param.into_inner(), &query).await {
                        Ok(item) => {
                            match item.to_document_automatically(
                                &self.uri.to_string(),
                                &query,
                                &req.uri().into(),
                            ) {
                                Ok(doc) => Ok(new_json_api_resp(StatusCode::OK).json(doc)),
                                Err(err) => Ok(error_to_response(err)),
                            }
                        },
                        Err(err) => Ok(error_to_response(err)),
                    }
                };

                fut.boxed_local().compat()
            },
            Err(err) => futures::future::ok(error_to_response(err)).boxed_local().compat(),
        }
    }

    pub fn fetch_relationship(
        self, param: web::Path<(String, String)>, req: HttpRequest,
    ) -> impl futures01::Future<Item = HttpResponse, Error = actix_web::Error> {
        if let Err(err_resp) = check_header(&self.jsonapi.version, &req.headers()) {
            return futures::future::ok(err_resp).boxed_local().compat();
        }
        match Query::from_uri(req.uri()) {
            Ok(query) => {
                let (id, related_field) = param.into_inner();
                let fut = async move {
                    match T::fetch_relationship(
                        &id,
                        &related_field,
                        &self.uri.to_string(),
                        &query,
                        &req.uri().into(),
                    )
                    .await
                    {
                        Ok(item) => Ok(new_json_api_resp(StatusCode::OK).json(item)),
                        Err(err) => Ok(error_to_response(err)),
                    }
                };

                fut.boxed_local().compat()
            },
            Err(err) => futures::future::ok(error_to_response(err)).boxed_local().compat(),
        }
    }

    pub fn fetch_related(
        self, param: web::Path<(String, String)>, req: HttpRequest,
    ) -> impl futures01::Future<Item = HttpResponse, Error = actix_web::Error> {
        if let Err(err_resp) = check_header(&self.jsonapi.version, &req.headers()) {
            return futures::future::ok(err_resp).boxed_local().compat();
        }

        match Query::from_uri(req.uri()) {
            Ok(query) => {
                let (id, related_field) = param.into_inner();
                let fut = async move {
                    match T::fetch_related(
                        &id,
                        &related_field,
                        &self.uri.to_string(),
                        &query,
                        &req.uri().into(),
                    )
                    .await
                    {
                        Ok(item) => Ok(new_json_api_resp(StatusCode::OK).json(item)),
                        Err(err) => Ok(error_to_response(err)),
                    }
                };
                fut.boxed_local().compat()
            },
            Err(err) => futures::future::ok(error_to_response(err)).boxed_local().compat(),
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
