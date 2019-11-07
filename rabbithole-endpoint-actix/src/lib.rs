use actix_web::http::StatusCode;
use actix_web::web;
use actix_web::{HttpRequest, HttpResponse};
use futures::{FutureExt, TryFutureExt};
use rabbithole::entity::SingleEntity;

use rabbithole::model::query::Query;
use rabbithole::operation::Fetching;
use serde::export::TryFrom;
use serde::Deserialize;
use serde_json::Value;
use std::marker::PhantomData;

#[derive(Debug, Deserialize, Clone)]
pub struct ActixSettingsModel {
    pub domain: String,
    pub suffix: String,
}

#[derive(Debug, Clone)]
pub struct ActixSettings<T>
where
    T: 'static + Fetching<Error = HttpResponse>,
{
    pub domain: url::Url,
    pub suffix: String,
    pub uri: url::Url,
    _data: PhantomData<T>,
}

impl<T> TryFrom<ActixSettingsModel> for ActixSettings<T>
where
    T: 'static + Fetching<Error = HttpResponse>,
{
    type Error = url::ParseError;

    fn try_from(value: ActixSettingsModel) -> Result<Self, Self::Error> {
        let ActixSettingsModel { domain, suffix } = value;
        let domain = domain.parse::<url::Url>()?;
        let uri = domain.join(&suffix)?;
        Ok(Self { domain, suffix, uri, _data: PhantomData })
    }
}

impl<T> ActixSettings<T>
where
    T: 'static + Fetching<Error = HttpResponse>,
{
    pub fn fetch_collection(
        self, req: HttpRequest,
    ) -> impl futures01::Future<Item = HttpResponse, Error = actix_web::Error> {
        if let Ok(query) = Query::from_uri(req.uri(), "CursorBased", "Rsql") {
            let fut = async move {
                let vec_res = T::fetch_collection(&query).await;
                match vec_res {
                    Ok(vec) => {
                        match T::vec_to_document(&vec, &self.uri.to_string(), &query).await {
                            Ok(doc) => Ok(HttpResponse::Ok().json(doc)),
                            Err(err) => Ok(err),
                        }
                    },
                    Err(err_resp) => Ok(err_resp),
                }
            };

            fut.boxed_local().compat()
        } else {
            async {
                Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(Value::String("Parser query error".into())))
            }
            .boxed_local()
            .compat()
        }
    }

    pub fn fetch_single(
        self, param: web::Path<String>, req: HttpRequest,
    ) -> impl futures01::Future<Item = HttpResponse, Error = actix_web::Error> {
        if let Ok(query) = Query::from_uri(req.uri(), "CursorBased", "Rsql") {
            let fut = async move {
                match T::fetch_single(&param.into_inner(), &query).await {
                    Ok(item) => {
                        match item.to_document_automatically(&self.uri.to_string(), &query) {
                            Ok(doc) => Ok(HttpResponse::Ok().json(doc)),
                            Err(err) => Ok(HttpResponse::InternalServerError()
                                .body(format!("err inner: {}", err))),
                        }
                    },
                    Err(err) => Ok(err),
                }
            };

            fut.boxed_local().compat()
        } else {
            async {
                Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(Value::String("Parser query error".into())))
            }
            .boxed_local()
            .compat()
        }
    }

    pub fn fetch_relationship(
        self, param: web::Path<(String, String)>, req: HttpRequest,
    ) -> impl futures01::Future<Item = HttpResponse, Error = actix_web::Error> {
        let (id, related_field) = param.into_inner();
        if let Ok(query) = Query::from_uri(req.uri(), "CursorBased", "Rsql") {
            let fut = async move {
                match T::fetch_relationship(&id, &related_field, &self.uri.to_string(), &query)
                    .await
                {
                    Ok(item) => Ok(HttpResponse::Ok().json(item)),
                    Err(err) => Ok(err),
                }
            };

            fut.boxed_local().compat()
        } else {
            async {
                Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(Value::String("Parser query error".into())))
            }
            .boxed_local()
            .compat()
        }
    }

    pub fn fetch_related(
        self, param: web::Path<(String, String)>, req: HttpRequest,
    ) -> impl futures01::Future<Item = HttpResponse, Error = actix_web::Error> {
        let (id, related_field) = param.into_inner();
        if let Ok(query) = Query::from_uri(req.uri(), "CursorBased", "Rsql") {
            let fut = async move {
                match T::fetch_related(&id, &related_field, &self.uri.to_string(), &query).await {
                    Ok(item) => Ok(HttpResponse::Ok().json(item)),
                    Err(err) => Ok(err),
                }
            };
            fut.boxed_local().compat()
        } else {
            async {
                Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(Value::String("Parser query error".into())))
            }
            .boxed_local()
            .compat()
        }
    }
}
