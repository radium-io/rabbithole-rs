use quote::quote;

pub fn generate_app(service: &syn::Path, ty: &str) -> proc_macro2::TokenStream {
    quote! {
        impl #service {
            pub fn actix_service() -> actix_web::Scope {
                use actix_web::{web, guard};
                web::scope(#ty)
                    .service(web::resource("")
                            .route(web::get().to_async(rabbithole_endpoint_actix::ActixSettings::fetch_collection::<Self>))
                            .route(web::post().to_async(rabbithole_endpoint_actix::ActixSettings::create::<Self>))
                            .route(web::delete().to_async(rabbithole_endpoint_actix::ActixSettings::delete_resource::<Self>))
                        )
                    .service(web::resource("/{id}")
                            .route(web::get().to_async( rabbithole_endpoint_actix::ActixSettings::fetch_single::<Self>))
                            .route(web::patch().to_async(rabbithole_endpoint_actix::ActixSettings::update_resource::<Self>))
                        )
                    .service(web::resource("/{id}/relationships/{related_fields}")
                            .route(web::get().to_async( rabbithole_endpoint_actix::ActixSettings::fetch_relationship::<Self>))
                            .route(web::patch().to_async(rabbithole_endpoint_actix::ActixSettings::replace_relationship::<Self>))
                            .route(web::post().to_async(rabbithole_endpoint_actix::ActixSettings::add_relationship::<Self>))
                            .route(web::delete().to_async(rabbithole_endpoint_actix::ActixSettings::remove_relationship::<Self>))
                        )
                    .service(web::resource("/{id}/{related_fields}")
                            .route(web::get().to_async( rabbithole_endpoint_actix::ActixSettings::fetch_related::<Self>))
                        )
            }
        }
    }
}
