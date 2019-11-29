use actix_http::encoding::Decoder;
use actix_http::Payload;
use actix_web::client::ClientResponse;
use rabbithole::model::document::Document;

/// https://jsonapi.org/format/#fetching-resources
pub mod integration_test;
#[macro_use]
pub mod paging;

async fn resp_to_doc(mut resp: ClientResponse<Decoder<Payload>>) -> Document {
    let bytes = resp.body().await.unwrap();
    let body = String::from_utf8(Vec::from(bytes.as_ref())).unwrap();
    serde_json::from_str(&body).unwrap()
}
