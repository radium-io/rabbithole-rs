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
                syn::Meta::List(syn::MetaList { path, nested, .. }) => {
                    if let (Some(field_ty), Some(syn::NestedMeta::Meta(nested_last))) =
                        (path.get_ident(), nested.last())
                    {
                        if let Some(_inner_type) = nested_last.path().get_ident() {
                            if field_ty == "to_many" {
                                return Ok(FieldType::ToMany);
                            } else if field_ty == "to_one" {
                                return Ok(FieldType::ToOne);
                            } else {
                                return Err(syn::Error::new_spanned(
                                    field_ty,
                                    EntityDecoratorError::InvalidUnitDecorator(
                                        field_ty.to_string(),
                                    ),
                                ));
                            }
                        } else {
                            return Err(syn::Error::new_spanned(
                                nested_last,
                                EntityDecoratorError::InvalidUnitDecorator(String::default()),
                            ));
                        }
                    }
                },
                syn::Meta::Path(syn::Path { segments, .. }) => {
                    if let Some(seg) = segments.last() {
                        let field_ty = &seg.ident;
                        if field_ty == "id" {
                            return Ok(FieldType::Id);
                        } else if field_ty == "to_many" {
                            let _inner_type = get_type(&item.ty, true)?;
                            return Ok(FieldType::ToMany);
                        } else if field_ty == "to_one" {
                            let _inner_type = get_type(&item.ty, false)?;
                            return Ok(FieldType::ToOne);
                        } else {
                            return Err(syn::Error::new_spanned(
                                field_ty,
                                EntityDecoratorError::InvalidParamDecorator(field_ty.to_string()),
                            ));
                        }
                    } else {
                        return Err(syn::Error::new_spanned(
                            meta_item,
                            EntityDecoratorError::InvalidParamDecorator(
                                meta_item.path().segments.to_token_stream().to_string(),
                            ),
                        ));
                    }
                },
                _ => {
                    return Err(syn::Error::new_spanned(
                        meta_item,
                        EntityDecoratorError::InvalidParamDecorator(
                            meta_item.path().segments.to_token_stream().to_string(),
                        ),
                    ))
                },
            }
        }
    }

    Ok(FieldType::Plain)
}

fn get_type(ty: &syn::Type, is_to_many: bool) -> syn::Result<&syn::Ident> {
    // Get zero- or first-layer of the Type
    let (outer_ty, inner_ty) =
        if let syn::Type::Path(syn::TypePath { path: syn::Path { segments, .. }, .. }) = ty {
            if let Some(syn::PathSegment { ident, arguments }) = segments.last() {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    args,
                    ..
                }) = arguments
                {
                    if let Some(syn::GenericArgument::Type(syn::Type::Path(inner_ty))) = args.last()
                    {
                        if let Some(inner_seg) = inner_ty.path.segments.last() {
                            if inner_seg.arguments.is_empty() {
                                (Some(ident), Some(&inner_seg.ident))
                            } else {
                                Default::default()
                            }
                        } else {
                            (Some(ident), None)
                        }
                    } else {
                        (Some(ident), None)
                    }
                } else {
                    (Some(ident), None)
                }
            } else {
                Default::default()
            }
        } else {
            Default::default()
        };

    let (invalid_vec, error_info): (&Vec<&str>, EntityDecoratorError) = if is_to_many {
        (&VALID_TO_MANY_WRAPPER, EntityDecoratorError::InvalidToManyType)
    } else {
        (&VALID_TO_ONE_WRAPPER, EntityDecoratorError::InvalidToOneType)
    };

    let err = Err(syn::Error::new_spanned(ty, error_info));

    match (outer_ty, inner_ty) {
        (Some(o), Some(i)) if invalid_vec.contains(&o.to_string().as_str()) => Ok(i),
        (Some(_), None) if is_to_many => err,
        (Some(o), None) => Ok(o),
        _ => err,
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FieldType {
    Id,
    ToOne,
    ToMany,
    Plain,
}
