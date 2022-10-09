mod api;
mod error;
mod one_response;
mod response;
mod uni_open_api;
mod uni_response;

use api::Apis;
use error::GeneratorResult;
use proc_macro::TokenStream;
use response::Responses;
use syn::{parse_macro_input, DeriveInput};
use uni_response::SUPPORT_STATUS;

#[proc_macro_derive(UniOpenApi)]
pub fn derive_uni_open_api(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as DeriveInput);
    uni_open_api::generate(&args)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro]
pub fn api(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Apis);
    api::generate(&args)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_derive(OneResponse, attributes(oai))]
pub fn derive_one_response(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as DeriveInput);
    match one_response::generate(&args) {
        Ok(stream) => stream.into(),
        Err(err) => err.write_errors().into(),
    }
}

#[proc_macro]
pub fn response(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Responses);
    response::generate(&args)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[doc(hidden)]
#[proc_macro]
pub fn generate_define_uni_response(_: TokenStream) -> TokenStream {
    uni_response::generate().into()
}
