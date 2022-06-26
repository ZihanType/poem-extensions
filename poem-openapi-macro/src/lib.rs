#[cfg(feature = "error")]
mod error;
#[cfg(feature = "one_response")]
mod one_response;
#[cfg(feature = "response")]
mod response;
#[cfg(feature = "uni_open_api")]
mod uni_open_api;

#[cfg(feature = "error")]
use error::GeneratorResult;
#[cfg(feature = "macros")]
use proc_macro::TokenStream;
#[cfg(feature = "response")]
use response::Responses;
#[cfg(feature = "macros")]
use syn::{parse_macro_input, DeriveInput};

#[cfg(feature = "uni_open_api")]
#[proc_macro_derive(UniOpenApi)]
pub fn derive_uni_open_api(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as DeriveInput);
    uni_open_api::generate(&args)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[cfg(feature = "one_response")]
#[proc_macro_derive(OneResponse, attributes(oai))]
pub fn derive_one_response(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as DeriveInput);
    match one_response::generate(&args) {
        Ok(stream) => stream.into(),
        Err(err) => err.write_errors().into(),
    }
}

#[cfg(feature = "response")]
#[proc_macro]
pub fn response(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Responses);
    response::generate(&args)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
