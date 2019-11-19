use quote::quote;

pub fn generate_app(service: &syn::Path, ty: &str) -> proc_macro2::TokenStream {
    quote! {
        impl #service {
            pub fn actix_service() -> actix_web::Scope {
                use actix_web::{web, guard};
                web::scope(#ty)
                    .service(web::resource("")
                            .route(web::get().to_async(move |req, service: web::Data<std::sync::Arc<futures::lock::Mutex<#service>>>, actix_operation: web::Data<rabbithole_endpoint_actix::ActixSettings<Self>>| rabbithole_endpoint_actix::ActixSettings::<Self>::fetch_collection(actix_operation.into_inner(), service, req)))
                            .route(web::post().to_async(move |body, req, service: web::Data<std::sync::Arc<futures::lock::Mutex<#service>>>, actix_operation: web::Data<rabbithole_endpoint_actix::ActixSettings<Self>>| rabbithole_endpoint_actix::ActixSettings::<Self>::create(actix_operation.into_inner(), service, req, body)))
                            .route(web::delete().to_async(move |params, req, service: web::Data<std::sync::Arc<futures::lock::Mutex<#service>>>, actix_operation: web::Data<rabbithole_endpoint_actix::ActixSettings<Self>>| rabbithole_endpoint_actix::ActixSettings::<Self>::delete_resource(actix_operation.into_inner(), service, params, req)))
                        )
                    .service(web::resource("/{id}")
                            .route(web::get().to_async(move |param, req, service: web::Data<std::sync::Arc<futures::lock::Mutex<#service>>>, actix_operation: web::Data<rabbithole_endpoint_actix::ActixSettings<Self>>| rabbithole_endpoint_actix::ActixSettings::<Self>::fetch_single(actix_operation.into_inner(), service, param, req)))
                            .route(web::patch().to_async(move |body, params, service: web::Data<std::sync::Arc<futures::lock::Mutex<#service>>>, req, actix_operation: web::Data<rabbithole_endpoint_actix::ActixSettings<Self>>| rabbithole_endpoint_actix::ActixSettings::<Self>::update_resource(actix_operation.into_inner(), service, req, params, body)))
                        )
                    .service(web::resource("/{id}/relationships/{related_fields}")
                            .route(web::get().to_async(move |param, req, service: web::Data<std::sync::Arc<futures::lock::Mutex<#service>>>, actix_operation: web::Data<rabbithole_endpoint_actix::ActixSettings<Self>>| rabbithole_endpoint_actix::ActixSettings::<Self>::fetch_relationship(actix_operation.into_inner(), service, param, req)))
                            .route(web::patch().to_async(move |param, body, req, service: web::Data<std::sync::Arc<futures::lock::Mutex<#service>>>, actix_operation: web::Data<rabbithole_endpoint_actix::ActixSettings<Self>>| rabbithole_endpoint_actix::ActixSettings::<Self>::replace_relationship(actix_operation.into_inner(), service, param, body, req)))
                            .route(web::post().to_async(move |param, body, req, service: web::Data<std::sync::Arc<futures::lock::Mutex<#service>>>, actix_operation: web::Data<rabbithole_endpoint_actix::ActixSettings<Self>>| rabbithole_endpoint_actix::ActixSettings::<Self>::add_relationship(actix_operation.into_inner(), service, param, body, req)))
                            .route(web::delete().to_async(move |param, body, req, service: web::Data<std::sync::Arc<futures::lock::Mutex<#service>>>, actix_operation: web::Data<rabbithole_endpoint_actix::ActixSettings<Self>>| rabbithole_endpoint_actix::ActixSettings::<Self>::remove_relationship(actix_operation.into_inner(), service, param, body, req)))
                        )
                    .service(web::resource("/{id}/{related_fields}")
                            .route(web::get().to_async(move |param, req, service: web::Data<std::sync::Arc<futures::lock::Mutex<#service>>>, actix_operation: web::Data<rabbithole_endpoint_actix::ActixSettings<Self>>| rabbithole_endpoint_actix::ActixSettings::<Self>::fetch_related(actix_operation.into_inner(), service, param, req)))
                        )
            }
        }
    }
}
