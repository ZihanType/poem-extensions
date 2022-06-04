use quote::quote;
use syn::DeriveInput;

pub(crate) fn generate(args: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &args.ident;

    let ret = quote! {
        impl ::poem_openapi::types::ParseFromParameter for #ident {
            fn parse_from_parameter(value: &str) -> ::poem_openapi::types::ParseResult<Self> {
                ::poem_openapi::types::ParseResult::Ok(::serde_qs::from_str(value)?)
            }
        }
    };

    Ok(ret)
}
