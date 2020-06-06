use super::ActixSettings;
use actix_service::{Service, Transform};
use actix_web::http::header;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, web, Error};
use futures::future::{ok, Ready};
use futures::Future;
use rabbithole::rule::RuleDispatcher;
use std::pin::Pin;
use std::task::{Context, Poll};

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct JsonApi;

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S> for JsonApi
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Error = Error;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    type InitError = ();
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Transform = JsonApiMiddleware<S>;

    fn new_transform(&self, service: S) -> Self::Future { ok(JsonApiMiddleware { service }) }
}

pub struct JsonApiMiddleware<S> {
    service: S,
}

impl<S, B> Service for JsonApiMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let settings: web::Data<ActixSettings> = req.app_data().unwrap();
        let api_version = &settings.get_ref().jsonapi.version;
        let headers = req.headers();
        let content_type = headers
            .get(header::CONTENT_TYPE)
            .map(|r| r.to_str().unwrap().to_string());
        let accept = headers
            .get(header::ACCEPT)
            .map(|r| r.to_str().unwrap().to_string());

        if let Err(e) = RuleDispatcher::ContentTypeMustBeJsonApi(api_version, &content_type) {
            let mut res =
                req.into_response(HttpResponse::UnsupportedMediaType().json(e).into_body());
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                rabbithole::JSON_API_HEADER.parse().unwrap(),
            );
            return Box::pin(ok(res));
        }

        if let Err(e) = RuleDispatcher::AcceptHeaderShouldBeJsonApi(api_version, &accept) {
            let mut res = req.into_response(HttpResponse::NotAcceptable().json(e).into_body());
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                rabbithole::JSON_API_HEADER.parse().unwrap(),
            );
            return Box::pin(ok(res));
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            res.headers_mut().insert(
                header::CONTENT_TYPE,
                rabbithole::JSON_API_HEADER.parse().unwrap(),
            );

            Ok(res)
        })
    }
}

use actix_web::HttpResponse;
