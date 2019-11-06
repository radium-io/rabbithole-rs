use actix_web::http::StatusCode;
use actix_web::web;
use actix_web::{HttpRequest, HttpResponse};
use futures::{FutureExt, TryFutureExt};
use rabbithole::entity::SingleEntity;

use rabbithole::model::query::Query;
use rabbithole::operation::Fetching;
use serde::export::PhantomData;
use serde_json::Value;

#[derive(Clone)]
pub struct ActixSettings<T> {
    pub uri: String,
    _data: PhantomData<T>,
}

impl<T> ActixSettings<T>
where
    T: 'static + Fetching<Error = HttpResponse>,
{
    pub fn new(uri: &str) -> ActixSettings<T> { Self { uri: uri.to_string(), _data: PhantomData } }

    pub fn fetch_collection(
        self, req: HttpRequest,
    ) -> impl futures01::Future<Item = HttpResponse, Error = actix_web::Error> {
        if let Ok(query) = Query::from_uri(req.uri(), "CursorBased", "Rsql") {
            let fut = async move {
                let vec_res = T::fetch_collection(&query).await;
                match vec_res {
                    Ok(vec) => match T::vec_to_document(&vec, &self.uri, &query).await {
                        Ok(doc) => Ok(HttpResponse::Ok().json(doc)),
                        Err(err) => Ok(err),
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
            let fut =
                async move {
                    match T::fetch_single(&param.into_inner(), &query).await {
                        Ok(item) => match item.to_document_automatically(&self.uri, &query) {
                            Ok(doc) => Ok(HttpResponse::Ok().json(doc)),
                            Err(err) => Ok(HttpResponse::InternalServerError()
                                .body(format!("err inner: {}", err))),
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
                match T::fetch_relationship(&id, &related_field, &self.uri, &query).await {
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
                match T::fetch_related(&id, &related_field, &self.uri, &query).await {
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
