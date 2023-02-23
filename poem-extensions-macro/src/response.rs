use std::collections::HashMap;

use proc_macro2::{Literal, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};

use crate::SUPPORT_STATUS;

pub(crate) struct Responses {
    responses: Vec<Response>,
}

struct Response {
    status_code: u16,
    response_type: syn::Type,
}

impl syn::parse::Parse for Responses {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut responses = vec![];

        while !input.is_empty() {
            let status_code: syn::LitInt = input.parse()?;
            input.parse::<syn::Token!(:)>()?;
            let response_type: syn::Type = input.parse()?;

            responses.push(Response {
                status_code: status_code.base10_parse::<u16>()?,
                response_type,
            });

            if !input.is_empty() {
                input.parse::<syn::Token!(,)>()?;
            }
        }

        Ok(Responses { responses })
    }
}

impl ToTokens for Responses {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.responses.iter().for_each(|response| {
            tokens.append(Literal::u16_unsuffixed(response.status_code));
            <syn::Token![:]>::default().to_tokens(tokens);
            response.response_type.to_tokens(tokens);
            <syn::Token![,]>::default().to_tokens(tokens);
        })
    }
}

pub(crate) fn generate(args: &Responses) -> syn::Result<TokenStream> {
    let (status_to_type, unsupport_status): (HashMap<&u16, &syn::Type>, HashMap<&u16, &syn::Type>) =
        args.responses
            .iter()
            .map(|r| (&r.status_code, &r.response_type))
            .partition(|(s, _)| SUPPORT_STATUS.contains(s));

    if !unsupport_status.is_empty() {
        return Err(syn::Error::new_spanned(
            args,
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
