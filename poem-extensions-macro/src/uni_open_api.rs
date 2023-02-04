use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Member};

type StructFields = syn::punctuated::Punctuated<syn::Field, syn::Token![,]>;

fn get_fields(args: &DeriveInput) -> syn::Result<&StructFields> {
    let struct_ident = &args.ident;

    match &args.data {
        syn::Data::Enum(_) => Err(syn::Error::new_spanned(struct_ident, "enum not supported")),
        syn::Data::Union(_) => Err(syn::Error::new_spanned(struct_ident, "union not supported")),
        syn::Data::Struct(ds) => match ds.fields {
            syn::Fields::Named(_) => Err(syn::Error::new_spanned(
                struct_ident,
                "named fields not supported",
            )),
            syn::Fields::Unit => Err(syn::Error::new_spanned(
                struct_ident,
                "unit struct not supported",
            )),
            syn::Fields::Unnamed(syn::FieldsUnnamed { ref unnamed, .. }) => Ok(unnamed),
        },
    }
}

pub(crate) fn generate(args: &DeriveInput) -> syn::Result<TokenStream> {
    let struct_ident = &args.ident;
    let (impl_generics, ty_generics, where_clause) = args.generics.split_for_impl();

    let (indexes, types): (Vec<Member>, Vec<&syn::Type>) = get_fields(args)?
        .iter()
        .map(|f| &f.ty)
        .enumerate()
        .map(|(idx, ty)| (Member::from(idx), ty))
        .unzip();

    let cap = types.len();

    let expand = quote! {
        impl #impl_generics ::poem_openapi::OpenApi for #struct_ident #ty_generics #where_clause {
            fn meta() -> ::std::vec::Vec<::poem_openapi::registry::MetaApi> {
                let mut metadata = ::std::vec::Vec::with_capacity(#cap);
                #(
                    metadata.extend(<#types as ::poem_openapi::OpenApi>::meta());
                )*
                metadata
            }

            fn register(registry: &mut ::poem_openapi::registry::Registry) {
                #(
                    <#types as ::poem_openapi::OpenApi>::register(registry);
                )*
            }

            fn add_routes(self, route_table: &mut ::std::collections::HashMap<::std::string::String, ::std::collections::HashMap<::poem::http::Method, ::poem::endpoint::BoxEndpoint<'static>>>) {
                #(
                    let route = self.#indexes.add_routes(route_table);
                )*
            }
        }
    };

    Ok(expand)
}
