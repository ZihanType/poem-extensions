use crate::SUPPORT_STATUS;
use proc_macro2::{Literal, Punct, Spacing};
use quote::{quote, ToTokens, TokenStreamExt};
use std::collections::HashMap;

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
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.responses.iter().for_each(|response| {
            tokens.append(Literal::u16_unsuffixed(response.status_code));
            tokens.append(Punct::new(':', Spacing::Alone));
            response.response_type.to_tokens(tokens);
            tokens.append(Punct::new(',', Spacing::Alone));
        })
    }
}

pub(crate) fn generate(args: &Responses) -> syn::Result<proc_macro2::TokenStream> {
    let status_to_type: HashMap<&u16, &syn::Type> = (&args.responses)
        .iter()
        .map(|r| (&r.status_code, &r.response_type))
        .collect();

    let unsupport_status: Vec<_> = status_to_type
        .keys()
        .filter(|s| !SUPPORT_STATUS.contains(s))
        .collect();

    if !unsupport_status.is_empty() {
        return Err(syn::Error::new_spanned(
            args,
            format!(
                "\n  support status code: {:?}\nunsupport status code: {:?}",
                SUPPORT_STATUS, unsupport_status
            ),
        ));
    }

    let response_types: Vec<_> = SUPPORT_STATUS
        .iter()
        .map(|status| match status_to_type.get(status) {
            Some(response_type) => {
                quote!(#response_type)
            }
            None => {
                quote!(::poem_openapi_response::Empty)
            }
        })
        .collect();

    let ret = quote! {
        ::poem_openapi_response::UniResponse<
            #(
                #response_types,
            )*
        >
    };

    Ok(ret)
}
