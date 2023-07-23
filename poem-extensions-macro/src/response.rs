use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    LitInt, Token, Type,
};

use crate::SUPPORT_STATUS;

pub(crate) struct Responses {
    responses: Punctuated<Response, Token![,]>,
}

impl Parse for Responses {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Responses {
            responses: input.parse_terminated(Response::parse, Token![,])?,
        })
    }
}

impl ToTokens for Responses {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.responses.to_tokens(tokens);
    }
}

struct Response {
    status_code: u16,
    colon_token: Token![:],
    response_type: Type,
}

impl Parse for Response {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            status_code: input.parse::<LitInt>()?.base10_parse::<u16>()?,
            colon_token: input.parse()?,
            response_type: input.parse()?,
        })
    }
}

impl ToTokens for Response {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.status_code.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.response_type.to_tokens(tokens);
    }
}

pub(crate) fn generate(args: Responses) -> syn::Result<TokenStream> {
    let (status_to_type, unsupport_status): (HashMap<_, _>, HashMap<_, _>) = args
        .responses
        .iter()
        .map(|r| {
            let Response {
                status_code,
                response_type,
                ..
            } = r;

            (status_code, response_type)
        })
        .partition(|(s, _)| SUPPORT_STATUS.contains(s));

    if !unsupport_status.is_empty() {
        return Err(syn::Error::new(
            args.span(),
            format!(
                "\n  support status code: {SUPPORT_STATUS:?}\nunsupport status code: {:?}",
                unsupport_status.keys(),
            ),
        ));
    }

    let response_types = SUPPORT_STATUS
        .iter()
        .map(|status| match status_to_type.get(status) {
            Some(response_type) => {
                quote!(#response_type)
            }
            None => {
                quote!(::poem_extensions::Empty)
            }
        });

    let expand = quote! {
        ::poem_extensions::UniResponse<
            #(
                #response_types,
            )*
        >
    };

    Ok(expand)
}
