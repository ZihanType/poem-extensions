use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(crate) const SUPPORT_STATUS: [u16; 60] = [
    100, 101, 102, // 1
    200, 201, 202, 203, 204, 205, 206, 207, 208, 226, // 2
    300, 301, 302, 303, 304, 305, 307, 308, // 3
    400, 401, 402, 403, 404, 405, 406, 407, 408, 409, 410, 411, 412, 413, // 4
    414, 415, 416, 417, 418, 421, 422, 423, 424, 426, 428, 429, 431, 451, // 4
    500, 501, 502, 503, 504, 505, 506, 507, 508, 510, 511, //5
];

pub(crate) fn generate() -> TokenStream {
    let generics = SUPPORT_STATUS
        .iter()
        .map(|s| format_ident!("T{}", s))
        .collect::<Vec<_>>();
    let status = SUPPORT_STATUS
        .iter()
        .map(|s| quote!(#s))
        .collect::<Vec<_>>();

    let expand = quote! {
        pub enum UniResponse<
            #(
                #generics = crate::Empty,
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
                #generics: ::poem_openapi::ApiResponse,
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

        fn meta_helper<T: ::poem_openapi::ApiResponse>(outer_responses: &mut ::std::vec::Vec<::poem_openapi::registry::MetaResponse>, outer_status: u16) {
            let mut inner_responses = T::meta().responses;

            // default empty response
            if inner_responses.is_empty() {
                return;
            }

            // only get first response in T responses
            if let Some(inner_status) = inner_responses.first().unwrap().status {
                if outer_status == inner_status {
                    outer_responses.push(inner_responses.swap_remove(0));
                } else {
                    panic!(
                        "status code of the first response in {} responses is not {}",
                        ::std::any::type_name::<T>(),
                        outer_status
                    );
                }
            } else {
                panic!(
                    "status code of the first response in {} responses is none",
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
