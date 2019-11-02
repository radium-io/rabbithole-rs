extern crate proc_macro;
#[macro_use]
extern crate thiserror;
#[macro_use]
extern crate lazy_static;

mod error;
mod field;

use crate::error::EntityDecoratorError;
use crate::field::{get_field_type, FieldType};
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

type FieldBundle<'a> =
    (&'a syn::Ident, Vec<&'a syn::Ident>, Vec<&'a syn::Ident>, Vec<&'a syn::Ident>);

#[proc_macro_derive(EntityDecorator, attributes(entity))]
pub fn derive(input: TokenStream) -> TokenStream {
    inner_derive(input).unwrap_or_else(|err| err.to_compile_error()).into()
}

fn inner_derive(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let ast: DeriveInput = syn::parse(input)?;
    let decorated_struct: &syn::Ident = &ast.ident;
    let struct_lifetime = &ast.generics;

    let entity_type = get_entity_type(&ast)?;

    let (id, attrs, to_ones, to_manys) = get_fields(&ast)?;

    let res = quote! {
        impl #struct_lifetime rabbithole::entity::Entity for #decorated_struct#struct_lifetime {
            fn ty(&self) -> std::option::Option<std::string::String> { std::option::Option::Some(#entity_type.to_string()) }
            fn id(&self) -> std::option::Option<std::string::String> { std::option::Option::Some(self.#id.to_string()) }

            fn attributes(&self) -> std::option::Option<rabbithole::model::resource::Attributes> {
                let mut attr_map: std::collections::HashMap<String, serde_json::Value> = std::default::Default::default();
                #(  if let Ok(json_value) = serde_json::to_value(self.#attrs.clone()) { attr_map.insert(stringify!(#attrs).to_string(), json_value); } )*
                Some(attr_map.into())
            }
            fn relationships(&self, uri: &str) -> rabbithole::RbhOptionRes<rabbithole::model::relationship::Relationships> {
                let mut relat_map: rabbithole::model::relationship::Relationships = std::default::Default::default();
                #(
                    if let (Some(relat_id), Some(links)) = (
                        self.#to_ones.to_resource_identifier(),
                        self.to_relationship_links(stringify!(#to_ones), uri)?,
                    ) {
                        let data = IdentifierData::Single(Some(relat_id));
                        let relat = Relationship { data, links, ..Default::default() };
                        relat_map.insert(stringify!(#to_ones).to_string(), relat);
                    }
                )*

                #(
                    if let Some(links) = self.to_relationship_links(stringify!(#to_manys), uri)? {
                        let mut relat_ids: ResourceIdentifiers = Default::default();
                        for item in &self.#to_manys {
                            if let Some(relat_id) = item.to_resource_identifier() {
                                relat_ids.push(relat_id);
                            }
                        }
                        let data = IdentifierData::Multiple(relat_ids);
                        let relat = Relationship { data, links, meta: Default::default() };
                        relat_map.insert(stringify!(#to_manys).to_string(), relat);
                    }
                )*

                Ok(Some(relat_map))
            }
            fn included(&self, uri: &str) -> rabbithole::RbhOptionRes<rabbithole::model::document::Included> {
                let mut included: rabbithole::model::document::Included = Default::default();
                #(
                    if let Some(res) = self.#to_ones.to_resource(uri)? {
                        included.push(res);
                    }
                )*
                #(
                    for item in &self.#to_manys {
                        if let Some(data) = item.to_resource(uri)? {
                            included.push(data);
                        }
                    }
                )*
                Ok(Some(included))
             }
        }
    };
    Ok(res)
}

fn get_meta(attrs: &[syn::Attribute]) -> syn::Result<Vec<syn::Meta>> {
    Ok(attrs
        .iter()
        .filter(|a| a.path.is_ident("entity"))
        .filter_map(|a| a.parse_meta().ok())
        .collect::<Vec<syn::Meta>>())
}

fn get_entity_type(ast: &syn::DeriveInput) -> syn::Result<String> {
    for meta in get_meta(&ast.attrs)? {
        if let syn::Meta::List(syn::MetaList { ref nested, .. }) = meta {
            if let Some(syn::NestedMeta::Meta(ref meta_item)) = nested.last() {
                if let syn::Meta::NameValue(syn::MetaNameValue {
                    path,
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = meta_item
                {
                    match path.segments.last() {
                        Some(syn::PathSegment { ident, .. }) if ident == "type" => {
                            return Ok(lit_str.value());
                        },
                        _ => {},
                    }
                }
            }
        }
    }

    Err(syn::Error::new_spanned(ast, EntityDecoratorError::InvalidEntityType))
}

fn get_fields(ast: &syn::DeriveInput) -> syn::Result<FieldBundle> {
    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        let mut id = None;
        let mut attrs = vec![];
        let mut to_ones = vec![];
        let mut to_manys = vec![];

        for n in named {
            let f: FieldType = get_field_type(n)?;
            match (f, n.ident.as_ref()) {
                (FieldType::Id, Some(ident)) if id.is_none() => id = Some(ident),
                (FieldType::Id, _) => {
                    return Err(syn::Error::new_spanned(n, EntityDecoratorError::DuplicatedId))
                },
                (FieldType::ToOne, Some(ident)) => to_ones.push(ident),
                (FieldType::ToMany, Some(ident)) => to_manys.push(ident),
                (FieldType::Plain, Some(ident)) => attrs.push(ident),
                _ => {
                    return Err(syn::Error::new_spanned(n, EntityDecoratorError::FieldWithoutName))
                },
            }
        }

        if let Some(id) = id {
            return Ok((id, attrs, to_ones, to_manys));
        }
    }
    Err(syn::Error::new_spanned(&ast.ident, EntityDecoratorError::InvalidEntityType))
}
