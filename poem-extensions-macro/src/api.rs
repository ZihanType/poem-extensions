use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Token,
};

pub(crate) struct Apis {
    apis: Punctuated<Ident, Token![,]>,
}

impl Parse for Apis {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Apis {
            apis: input.parse_terminated(Ident::parse, Token![,])?,
        })
    }
}

impl ToTokens for Apis {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.apis.to_tokens(tokens)
    }
}

pub(crate) fn generate(args: Apis) -> syn::Result<TokenStream> {
    let apis = args.apis.iter().collect::<Vec<_>>();

    let expand = quote! {
        {
            #[derive(::poem_extensions::UniOpenApi)]
            struct UniApi(#(#apis,)*);

            UniApi(#(#apis,)*)
        }
    };

    Ok(expand)
}
