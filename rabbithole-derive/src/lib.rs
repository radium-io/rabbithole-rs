extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::DeriveInput;

#[proc_macro_derive(ModelDecorator, attributes(model))]
pub fn derive(input: TokenStream) -> TokenStream {
    inner_derive(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn inner_derive(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let ast: DeriveInput = syn::parse(input)?;
    attrs_on_struct(&ast);
    eprintln!("=====  =====");
    extract_decorated_fields(&ast);
    let res = quote! {};
    Ok(res)
}

fn attrs_on_struct(ast: &syn::DeriveInput) -> syn::Result<()> {
    for attr in &ast.attrs {
        eprintln!("attr: {:#?}", attr);
    }

    Ok(())
}

fn extract_decorated_fields(ast: &syn::DeriveInput) -> syn::Result<()> {
    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        eprintln!("extract_decorated_fields");

        for n in named {
            eprintln!("field: {:#?}", n);
        }
        Ok(())
    } else {
        Err(syn::Error::new(
            ast.span(),
            "This macro can only handle Named Structs",
        ))
    }
}
