use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

pub(crate) const SUPPORT_STATUS: [u16; 60] = [
    100, 101, 102, // 1xx
    200, 201, 202, 203, 204, 205, 206, 207, 208, 226, // 2xx
    300, 301, 302, 303, 304, 305, 307, 308, // 3xx
    400, 401, 402, 403, 404, 405, 406, 407, 408, 409, 410, 411, 412, 413, // 4xx
    414, 415, 416, 417, 418, 421, 422, 423, 424, 426, 428, 429, 431, 451, // 4xx
    500, 501, 502, 503, 504, 505, 506, 507, 508, 510, 511, // 5xx
];

pub(crate) fn generate() -> TokenStream {
    let (generics, status): (Vec<Ident>, Vec<TokenStream>) = SUPPORT_STATUS
        .iter()
        .map(|s| (format_ident!("T{}", s), quote!(#s)))
        .unzip();

    let expand = quote! {
        pub enum UniResponse<
            #(
                #generics,
            )*
        > {
            #(
                #generics(#generics),
            )*
        }

        impl<#(#generics,)*> ::poem_openapi::ApiResponse
            for UniResponse<#(#generics,)*>
        where
            #(
                #generics: ::poem_openapi::ApiResponse + 'static,
            )*
        {
            const BAD_REQUEST_HANDLER: bool = false;

            fn meta() -> ::poem_openapi::registry::MetaResponses {
                let mut responses = ::std::vec![];

                #(
                    meta_helper::<#generics>(&mut responses, #status);
                )*

                ::poem_openapi::registry::MetaResponses { responses }
            }

            fn register(registry: &mut ::poem_openapi::registry::Registry) {
                #(
                    #generics::register(registry);
                )*
            }
        }

        fn meta_helper<T: ::poem_openapi::ApiResponse + 'static>(
            outer_responses: &mut ::std::vec::Vec<::poem_openapi::registry::MetaResponse>,
            outer_status: u16,
        ) {
            let mut inner_responses = T::meta().responses;

            // default empty response
            if ::std::any::TypeId::of::<T>() == ::std::any::TypeId::of::<crate::Empty>() {
                return;
            }

            if inner_responses.is_empty() {
                panic!("{} meta responses is empty", ::std::any::type_name::<T>());
            }

            // only get first meta response in T meta responses
            if let Some(inner_status) = inner_responses.first().unwrap().status {
                if outer_status == inner_status {
                    outer_responses.push(inner_responses.swap_remove(0));
                } else {
                    panic!(
                        "status code of the first meta response in {} meta responses is not {}",
                        ::std::any::type_name::<T>(),
                        outer_status
                    );
                }
            } else {
                panic!(
                    "status code of the first meta response in {} meta responses is none",
                    ::std::any::type_name::<T>()
                );
            }
        }

        impl<#(#generics,)*> ::poem::web::IntoResponse
            for UniResponse<#(#generics,)*>
        where
            #(
                #generics: ::poem::web::IntoResponse,
            )*
        {
            fn into_response(self) -> ::poem::Response {
                match self {
                    #(
                        UniResponse::#generics(t) => t.into_response(),
                    )*
                }
            }
        }
    };

    expand
}
