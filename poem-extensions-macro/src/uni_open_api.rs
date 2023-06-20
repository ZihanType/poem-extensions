use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Ident, Member, Type};

fn get_fields(struct_ident: &Ident, data: Data) -> syn::Result<(Vec<Member>, Vec<Type>)> {
    let fields = match data {
        Data::Enum(_) => return Err(syn::Error::new_spanned(struct_ident, "enum not supported")),
        Data::Union(_) => return Err(syn::Error::new_spanned(struct_ident, "union not supported")),
        Data::Struct(ds) => match ds.fields {
            Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    struct_ident,
                    "unit struct not supported",
                ))
            }
            Fields::Named(FieldsNamed { named, .. }) => named,
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => unnamed,
        },
    };

    let fields = fields
        .into_iter()
        .enumerate()
        .map(|(idx, f)| {
            let member = match f.ident {
                Some(ident) => Member::from(ident),
                None => Member::from(idx),
            };

            (member, f.ty)
        })
        .unzip();

    Ok(fields)
}

pub(crate) fn generate(args: DeriveInput) -> syn::Result<TokenStream> {
    let DeriveInput {
        ident: struct_ident,
        generics,
        data,
        ..
    } = args;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let (members, types) = get_fields(&struct_ident, data)?;

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
                    let route = self.#members.add_routes(route_table);
                )*
            }
        }
    };

    Ok(expand)
}
