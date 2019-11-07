use quote::quote;

pub fn generate_app(
    entity_ident: &syn::Ident, ty: &str, _to_ones: &[&syn::Ident], _to_manys: &[&syn::Ident],
) -> proc_macro2::TokenStream {
    quote! {
        impl #entity_ident {
            pub fn actix_service() -> actix_web::Scope {
                use actix_web::{web, guard};
                web::scope(#ty)
                    .service(web::resource("")
                        .route(web::get().to_async(move |req, actix_fetching: web::Data<rabbithole_endpoint_actix::ActixSettings<Self>>| actix_fetching.get_ref().clone().fetch_collection(req))))
                    .service(web::resource("/{id}")
                        .route(web::get().to_async(move |param, req, actix_fetching: web::Data<rabbithole_endpoint_actix::ActixSettings<Self>>| actix_fetching.get_ref().clone().fetch_single(param, req))))
                    .service(web::resource("/{id}/relationships/{related_fields}")
                        .route(web::get().to_async(move |param, req, actix_fetching: web::Data<rabbithole_endpoint_actix::ActixSettings<Self>>| actix_fetching.get_ref().clone().fetch_relationship(param, req))))
                    .service(web::resource("/{id}/{related_fields}")
                        .route(web::get().to_async(move |param, req, actix_fetching: web::Data<rabbithole_endpoint_actix::ActixSettings<Self>>| actix_fetching.get_ref().clone().fetch_related(param, req))))
            }
        }
    }
}
