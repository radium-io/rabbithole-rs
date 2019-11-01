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
use std::iter::FromIterator;
use syn::DeriveInput;

type FieldBundle<'a> = (
    &'a syn::Ident,
    Vec<&'a syn::Ident>,
    Vec<(syn::Ident, &'a syn::Ident)>,
    Vec<(syn::Ident, &'a syn::Ident)>,
);

#[proc_macro_derive(EntityDecorator, attributes(entity))]
pub fn derive(input: TokenStream) -> TokenStream {
    inner_derive(input).unwrap_or_else(|err| err.to_compile_error()).into()
}

fn to_ones_included(to_ones: &[(syn::Ident, &syn::Ident)]) -> proc_macro2::TokenStream {
    proc_macro2::TokenStream::from_iter(
    to_ones
        .iter()
        .map(|(ref inner_ty, field_name)| {
            quote! {
                if let Some(field) = &self.#field_name.to_entity()
                {
                    included.push(field.to_resource(uri)?);
                }
            }
        })
    )
}

fn to_ones_trait

fn to_ones_relat(to_ones: &[(syn::Ident, &syn::Ident)]) -> proc_macro2::TokenStream {
    proc_macro2::TokenStream::from_iter(
        to_ones
            .iter()
            .map(|(ref inner_ty, field_name)| {
                quote! {
                if let Some(field) = &self.#field_name.to_entity()
                {
                    let relat_id = field.to_resource_identifier();
                    let links = field.to_relationship_links(stringify!(#field_name), uri)?;
                    let data = rabbithole::model::resource::IdentifierData::Single(Some(relat_id));
                    let relat = rabbithole::model::relationship::Relationship { data, links, ..Default::default() };
                    relat_map.insert(stringify!(#field_name).to_string(), relat);

                }
            }
            })
    )
}

//fn to_manys_included(to_manys: &[(&syn::Ident, &syn::Ident)]) -> proc_macro2::TokenStream {
//    proc_macro2::TokenStream::from_iter(
//        to_manys
//            .iter()
//            .map(|(&inner_ty, &field_name)| {
//                quote! {
//                    for item in &self.#field_name {
//                        if let Some(field) = unsafe { std::mem::transmute::<_, &dyn EntityExt<T = #inner_ty>>(&self.#field_name) }.to_entity()
//                        {
//                            included.push(field.to_resource(uri)?);
//                        }
//                    }
//                }
//            })
//    )
//}

fn inner_derive(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let ast: DeriveInput = syn::parse(input)?;
    let decorated_struct: &syn::Ident = &ast.ident;
    let entity_type = get_entity_type(&ast)?;
    let (id, attrs, to_ones, _to_manys) = get_fields(&ast)?;
    let to_ones_included = to_ones_included(&to_ones);
    let to_ones_relat = to_ones_relat(&to_ones);

    let res = quote! {
            impl rabbithole::entity::Entity for #decorated_struct {
                fn ty(&self) -> String { #entity_type.to_string() }
                fn id(&self) -> String { self.#id.to_string() }
                fn attributes(&self) -> rabbithole::model::resource::Attributes {
                    let mut attr_map: std::collections::HashMap<String, serde_json::Value> = std::default::Default::default();
                    #(  if let Ok(json_value) = serde_json::to_value(self.#attrs.clone()) { attr_map.insert(stringify!(#attrs).to_string(), json_value); } )*
                    attr_map.into()
                }
                fn relationships(&self, uri: &str) -> rabbithole::RbhResult<rabbithole::model::relationship::Relationships> {
                    use rabbithole::entity::EntityExt;

                    let mut relat_map: rabbithole::model::relationship::Relationships = std::default::Default::default();
                    #to_ones_relat

    //                #(
    //                    let mut relat_ids: rabbithole::model::resource::ResourceIdentifiers = Default::default();
    //                    let links = self.to_relationship_links(stringify!(#to_manys), uri)?;
    //                    for item in &self.#to_manys {
    //                        if let Some(relat_id) = item.to_entity().to_resource_identifier() {
    //                            relat_ids.push(relat_id);
    //                        }
    //                    }
    //                    let data = rabbithole::model::resource::IdentifierData::Multiple(relat_ids);
    //                    let relat = rabbithole::model::relationship::Relationship { data, links, ..Default::default() };
    //                    relat_map.insert(stringify!(#to_manys).to_string(), relat);
    //                )*

                    Ok(relat_map)
                }
                fn included(&self, uri: &str) -> rabbithole::RbhResult<rabbithole::model::document::Included> {
                    use rabbithole::entity::EntityExt;
                    let mut included: rabbithole::model::document::Included = Default::default();
                    #to_ones_included
    //                #(
    //                    for item in &self.#to_manys {
    //                        if let Some(data) = item.to_entity().to_resource(uri)? {
    //                            included.push(data);
    //                        }
    //                    }
    //                )*

                    Ok(included)
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
                (FieldType::ToOne(inner_ty), Some(ident)) => to_ones.push((inner_ty, ident)),
                (FieldType::ToMany(inner_ty), Some(ident)) => to_manys.push((inner_ty, ident)),
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
