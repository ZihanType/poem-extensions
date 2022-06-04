mod query_object;
mod uni_open_api;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(UniOpenApi)]
pub fn derive_uni_open_api(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as DeriveInput);
    uni_open_api::generate(&args)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_derive(QueryObject)]
pub fn derive_object(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as DeriveInput);
    query_object::generate(&args)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
