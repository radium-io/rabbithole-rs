use quote::quote;

pub fn generate_server(
    entity_ident: &syn::Ident, ty: &str, _to_ones: &[&syn::Ident], _to_manys: &[&syn::Ident],
) -> proc_macro2::TokenStream {
    quote! {
        impl #entity_ident {
            pub fn actix_server<F, I, S, B>(domain: url::Url, suffix: &str) -> actix_web::HttpServer<F, I, S, B>
            where
                F: Fn() -> I + Send + Clone + 'static,
                I: actix_service::IntoNewService<S>,
                S: actix_service::NewService<Config = actix_service_config::ServerConfig, Request = actix_http::request::Request>,
                S::Error: Into<actix_http::error::Error>,
                S::InitError: std::fmt::Debug,
                S::Response: Into<actix_http::response::Response<B>>,
                S::Service: 'static,
                B: actix_http::body::MessageBody,{
                let uri: &str = domain.join(suffix).unwrap().as_str();
                actix_web::HttpServer::new( move ||
                    actix_web::app::App::new()
                        .data(rabbithole_endpoint_actix::ActixSettings::<#entity_ident>::new(uri))
                        .route(
                            format!("{}/{}", suffix, #ty),
                            actix_web::web::get().to_async(
                                move |param, req, actix_fetching: actix_web::web::Data<rabbithole_endpoint_actix::ActixSettings::<#entity_ident>>| {
                                    actix_fetching.get_ref().clone().fetch_collection(req)
                                }
                            )
                        )
                )
            }
        }
    }
}
