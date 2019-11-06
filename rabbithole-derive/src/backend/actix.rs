use quote::quote;

pub fn generate_app(
    entity_ident: &syn::Ident, ty: &str, _to_ones: &[&syn::Ident], _to_manys: &[&syn::Ident],
) -> proc_macro2::TokenStream {
    quote! {
        impl #entity_ident {
            pub fn actix_app(domain: url::Url, suffix: &str) -> actix_web::Scope {
                use actix_web::web;
                let uri: &str = domain.join(suffix).unwrap().as_str();
                web::scope(suffix)
                    .service(web::resource(#ty)
                        .route(web::get().to_async(move |req, actix_fetching: web::Data<rabbithole_endpoint_actix::ActixSettings<Human>>| actix_fetching.get_ref().clone().fetch_collection(req))))
                    .service(web::resource(&format!("{}/{{id}}", #ty))
                        .route(web::get().to_async(move |param, req, actix_fetching: web::Data<rabbithole_endpoint_actix::ActixSettings<Human>>| actix_fetching.get_ref().clone().fetch_single(param, req))))
                    .service(web::resource(&format!("{}/{{id}}/relationships/{{related_fields}}", #ty))
                        .route(web::get().to_async(move |param, req, actix_fetching: web::Data<ActixSettings<Human>>| actix_fetching.get_ref().clone().fetch_relationship(param, req))))
                    .service(web::resource(&format!("{}/{{id}}/{{related_fields}}", #ty))
                        .route(web::get().to_async(move |param, req, actix_fetching: web::Data<ActixSettings<Human>>| actix_fetching.get_ref().clone().fetch_related(param, req))))
            }
        }
    }
}
