use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Member};

#[proc_macro_derive(UniOpenApi)]
pub fn derive(input: TokenStream) -> TokenStream {
    let st = parse_macro_input!(input as DeriveInput);
    match generate(&st) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

type StructFields = syn::punctuated::Punctuated<syn::Field, syn::Token![,]>;

fn get_fields(st: &DeriveInput) -> syn::Result<&StructFields> {
    match &st.data {
        syn::Data::Enum(_) => Err(syn::Error::new_spanned(st, "enum not supported")),
        syn::Data::Union(_) => Err(syn::Error::new_spanned(st, "union not supported")),
        syn::Data::Struct(ds) => match ds.fields {
            syn::Fields::Named(_) => Err(syn::Error::new_spanned(st, "named fields not supported")),
            syn::Fields::Unit => Err(syn::Error::new_spanned(st, "unit struct not supported")),
            syn::Fields::Unnamed(syn::FieldsUnnamed { ref unnamed, .. }) => Ok(unnamed),
        },
    }
}

fn generate(st: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &st.ident;

    let types = get_fields(st)?.iter().map(|f| &f.ty).collect::<Vec<_>>();
    let idx = types
        .iter()
        .enumerate()
        .map(|a| Member::from(a.0))
        .collect::<Vec<_>>();

    let cap = types.len();

    let ret = quote! {
        impl ::poem_openapi::OpenApi for #struct_name {
            fn meta() -> ::std::vec::Vec<::poem_openapi::registry::MetaApi> {
                let mut metadata = ::std::vec::Vec::with_capacity(#cap);
                #(
                    metadata.extend(#types::meta());
                )*
                metadata
            }

            fn register(registry: &mut ::poem_openapi::registry::Registry) {
                #(
                    #types::register(registry);
                )*
            }

            fn add_routes(self, route: ::poem::Route) -> ::poem::Route {
                #(
                    let route = self.#idx.add_routes(route);
                )*
                route
            }
        }
    };

    Ok(ret)
}
