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

pub(crate) fn generate(args: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_ident = &args.ident;
    let (impl_generics, ty_generics, where_clause) = args.generics.split_for_impl();

    let types = get_fields(args)?.iter().map(|f| &f.ty).collect::<Vec<_>>();
    let idx = types
        .iter()
        .enumerate()
        .map(|a| Member::from(a.0))
        .collect::<Vec<_>>();

    let cap = types.len();

    let ret = quote! {
        impl #impl_generics ::poem_openapi::OpenApi for #struct_ident #ty_generics #where_clause {
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
