use proc_macro2::{Punct, Spacing, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};

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
            tokens.append(Punct::new(',', Spacing::Alone));
        })
    }
}

pub(crate) fn generate(args: &Apis) -> syn::Result<TokenStream> {
    let apis: Vec<TokenStream> = args.apis.iter().map(|id| quote!(#id)).collect();

    let expand = quote! {
        {
            #[derive(::poem_extensions::UniOpenApi)]
            struct UniApi(#(#apis,)*);

            UniApi(#(#apis,)*)
        }
    };

    Ok(expand)
}
