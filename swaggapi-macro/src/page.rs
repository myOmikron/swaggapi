use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse2, Fields, ItemStruct};

pub fn page(input: TokenStream) -> TokenStream {
    let ItemStruct {
        attrs,
        vis: _,
        struct_token: _,
        ident,
        generics: _,
        fields,
        semi_token: _,
    } = match parse2(input) {
        Ok(s) => s,
        Err(err) => return err.into_compile_error(),
    };

    if !matches!(&fields, Fields::Unit) {
        return quote_spanned! {fields.span()=>
            compile_error!("Expected unit struct");
        };
    }

    quote! {
        impl SwaggapiPage for #ident {
            fn builder() -> &'static SwaggapiPageBuilder {
                static BUILDER: SwaggapiPageBuilder = SwaggapiPageBuilder::new();
                &BUILDER
            }
        }
    }
}