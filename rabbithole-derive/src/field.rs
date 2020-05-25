use crate::error::EntityDecoratorError;
use crate::get_meta;
use quote::ToTokens;

lazy_static! {
    static ref VALID_TO_ONE_WRAPPER: Vec<&'static str> = vec!["Option", "Box"];
    static ref VALID_TO_MANY_WRAPPER: Vec<&'static str> = vec!["Vec", "HashSet"];
}

pub(crate) fn get_field_type(item: &syn::Field) -> syn::Result<FieldType> {
    if let Some(syn::Meta::List(syn::MetaList { ref nested, .. })) = get_meta(&item.attrs)?.last() {
        if let Some(syn::NestedMeta::Meta(ref meta_item)) = nested.last() {
            match meta_item {
                syn::Meta::Path(syn::Path { segments, .. }) => {
                    if let Some(seg) = segments.last() {
                        let field_ty = &seg.ident;
                        if field_ty == "id" {
                            return Ok(FieldType::Id);
                        } else if field_ty == "to_many" {
                            return Ok(FieldType::ToMany);
                        } else if field_ty == "to_one" {
                            return Ok(FieldType::ToOne);
                        } else {
                            return Err(syn::Error::new_spanned(
                                field_ty,
                                EntityDecoratorError::InvalidUnitDecorator(field_ty.to_string()),
                            ));
                        }
                    } else {
                        return Err(syn::Error::new_spanned(
                            meta_item,
                            EntityDecoratorError::InvalidUnitDecorator(
                                meta_item.path().segments.to_token_stream().to_string(),
                            ),
                        ));
                    }
                },
                _ => {
                    return Err(syn::Error::new_spanned(
                        meta_item,
                        EntityDecoratorError::InvalidUnitDecorator(
                            meta_item.path().segments.to_token_stream().to_string(),
                        ),
                    ));
                },
            }
        }
    }

    Ok(FieldType::Plain)
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FieldType {
    Id,
    ToOne,
    ToMany,
    Plain,
}
