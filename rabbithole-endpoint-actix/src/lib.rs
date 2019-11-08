pub mod settings;

use actix_web::http::{header, HeaderMap, StatusCode};
use actix_web::web;
use actix_web::{HttpRequest, HttpResponse};
use futures::{FutureExt, TryFutureExt};
use rabbithole::entity::SingleEntity;

use crate::settings::{ActixSettingsModel, JsonApiSettings};
use actix_web::dev::HttpResponseBuilder;

use rabbithole::error::RabbitholeError;
use rabbithole::model::query::Query;
use rabbithole::model::version::JsonApiVersion;
use rabbithole::operation::Fetching;
use rabbithole::rule::RuleDispatcher;
use rabbithole::JSON_API_HEADER;
use serde::export::TryFrom;
use serde_json::Value;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct ActixSettings<T>
where
    T: 'static + Fetching<Error = HttpResponse>,
{
    pub domain: url::Url,
    pub suffix: String,
    pub uri: url::Url,
    pub jsonapi: JsonApiSettings,
    _data: PhantomData<T>,
}

impl<T> TryFrom<ActixSettingsModel> for ActixSettings<T>
where
    T: 'static + Fetching<Error = HttpResponse>,
{
    type Error = url::ParseError;

    fn try_from(value: ActixSettingsModel) -> Result<Self, Self::Error> {
        let ActixSettingsModel { domain, suffix, jsonapi } = value;
        let domain = domain.parse::<url::Url>()?;
        let uri = domain.join(&suffix)?;
        Ok(Self { domain, suffix, uri, jsonapi, _data: PhantomData })
    }
}

impl<T> ActixSettings<T>
where
    T: 'static + Fetching<Error = HttpResponse>,
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
                                Err(err) => Ok(err),
                            }
                        },
                        Err(err_resp) => Ok(err_resp),
                    }
                };

                fut.boxed_local().compat()
            },
            Err(err) => futures::future::ok(query_parsing_error_resp(err)).boxed_local().compat(),
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
                        Ok(item) => match item.to_document_automatically(
                            &self.uri.to_string(),
                            &query,
                            &req.uri().into(),
                        ) {
                            Ok(doc) => Ok(new_json_api_resp(StatusCode::OK).json(doc)),
                            Err(err) => Ok(new_json_api_resp(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(format!("err inner: {}", err))),
                        },
                        Err(err) => Ok(err),
                    }
                };

                fut.boxed_local().compat()
            },
            Err(err) => futures::future::ok(query_parsing_error_resp(err)).boxed_local().compat(),
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
                    match T::fetch_relationship(&id, &related_field, &self.uri.to_string(), &query)
                        .await
                    {
                        Ok(item) => Ok(new_json_api_resp(StatusCode::OK).json(item)),
                        Err(err) => Ok(err),
                    }
                };

                fut.boxed_local().compat()
            },
            Err(err) => futures::future::ok(query_parsing_error_resp(err)).boxed_local().compat(),
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
                        Err(err) => Ok(err),
                    }
                };
                fut.boxed_local().compat()
            },
            Err(err) => futures::future::ok(query_parsing_error_resp(err)).boxed_local().compat(),
        }
    }
}

fn query_parsing_error_resp(err: RabbitholeError) -> HttpResponse {
    new_json_api_resp(StatusCode::BAD_REQUEST).json(Value::String(err.to_string()))
}

fn check_header(api_version: &JsonApiVersion, headers: &HeaderMap) -> Result<(), HttpResponse> {
    let content_type = headers.get(header::CONTENT_TYPE).map(|r| r.to_str().unwrap().to_string());
    let accept = headers.get(header::ACCEPT).map(|r| r.to_str().unwrap().to_string());
    if let Err(err) = RuleDispatcher::ContentTypeMustBeJsonApi(api_version, &content_type) {
        return Err(new_json_api_resp(StatusCode::from_u16(err).unwrap()).finish());
    }
    if let Err(err) = RuleDispatcher::AcceptHeaderShouldBeJsonApi(api_version, &accept) {
        return Err(new_json_api_resp(StatusCode::from_u16(err).unwrap()).finish());
    }

    Ok(())
}

fn new_json_api_resp(status_code: StatusCode) -> HttpResponseBuilder {
    let mut resp = HttpResponse::build(status_code);
    resp.set_header(header::CONTENT_TYPE, JSON_API_HEADER);
    resp
}
