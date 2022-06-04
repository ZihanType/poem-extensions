use quote::quote;
use syn::{DeriveInput, Member};

type StructFields = syn::punctuated::Punctuated<syn::Field, syn::Token![,]>;

fn get_fields(args: &DeriveInput) -> syn::Result<&StructFields> {
    match &args.data {
        syn::Data::Enum(_) => Err(syn::Error::new_spanned(args, "enum not supported")),
        syn::Data::Union(_) => Err(syn::Error::new_spanned(args, "union not supported")),
        syn::Data::Struct(ds) => match ds.fields {
            syn::Fields::Named(_) => {
                Err(syn::Error::new_spanned(args, "named fields not supported"))
            }
            syn::Fields::Unit => Err(syn::Error::new_spanned(args, "unit struct not supported")),
            syn::Fields::Unnamed(syn::FieldsUnnamed { ref unnamed, .. }) => Ok(unnamed),
        },
    }
}

pub(crate) fn generate(args: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &args.ident;

    let types = get_fields(args)?.iter().map(|f| &f.ty).collect::<Vec<_>>();
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
