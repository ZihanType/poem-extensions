use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub(crate) struct Apis {
    apis: Vec<syn::Ident>,
}

impl syn::parse::Parse for Apis {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut apis = vec![];

        while !input.is_empty() {
            let api: syn::Ident = input.parse()?;
            apis.push(api);

            if !input.is_empty() {
                input.parse::<syn::Token!(,)>()?;
            }
        }

        Ok(Apis { apis })
    }
}

impl ToTokens for Apis {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.apis.iter().for_each(|api| {
            api.to_tokens(tokens);
            <syn::Token![,]>::default().to_tokens(tokens);
        })
    }
}

pub(crate) fn generate(args: &Apis) -> syn::Result<TokenStream> {
    let apis = &args.apis;

    let expand = quote! {
        {
            #[derive(::poem_extensions::UniOpenApi)]
            struct UniApi(#(#apis,)*);

            UniApi(#(#apis,)*)
        }
    };

    Ok(expand)
}
