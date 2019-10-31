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
    let entity_type = get_entity_type(&ast)?;
    let (id, attrs, to_ones, to_manys) = get_fields(&ast)?;
    let res = quote! {
        impl rabbithole::entity::Entity for #decorated_struct {
            fn ty(&self) -> String { #entity_type.to_string() }
            fn id(&self) -> String { self.#id.to_string() }
            fn attributes(&self) -> rabbithole::model::resource::Attributes { Default::default() }
            fn links(&self) -> rabbithole::model::link::Links { Default::default() }
            fn relationships(&self) -> rabbithole::model::relationship::Relationships { Default::default() }
            fn included(&self) -> rabbithole::model::document::Included { Default::default() }
            fn meta(&self) -> rabbithole::model::Meta { Default::default() }
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
