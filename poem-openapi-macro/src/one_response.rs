use darling::{
    ast::{Data, Fields},
    util::{Ignored, SpannedValue},
    FromDeriveInput, FromField, FromMeta,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, DeriveInput, Generics, Lit, Meta, Path, Type};

use crate::GeneratorResult;

#[derive(FromMeta, Debug)]
struct ExtraHeader {
    name: String,

    #[darling(rename = "type")]
    ty: SpannedValue<String>,
    #[darling(default)]
    description: Option<String>,
    #[darling(default)]
    deprecated: bool,
}

#[derive(FromField, Debug)]
#[darling(attributes(oai), forward_attrs(doc))]
struct ResponseField {
    ty: Type,
    attrs: Vec<Attribute>,

    #[darling(default)]
    header: Option<String>,
    #[darling(default)]
    deprecated: bool,
}

#[derive(FromDeriveInput, Debug)]
#[darling(attributes(oai), forward_attrs(doc))]
struct ResponseArgs {
    ident: Ident,
    attrs: Vec<Attribute>,
    generics: Generics,
    data: Data<Ignored, ResponseField>,

    status: u16,
    #[darling(default)]
    bad_request_handler: Option<Path>,
    #[darling(default)]
    content_type: Option<String>,
    #[darling(default, multiple, rename = "header")]
    headers: Vec<ExtraHeader>,
}

pub(crate) fn generate(args: &DeriveInput) -> GeneratorResult<TokenStream> {
    let args: ResponseArgs = ResponseArgs::from_derive_input(args)?;
    let (impl_generics, ty_generics, where_clause) = args.generics.split_for_impl();
    let struct_ident = &args.ident;
    let status = get_status(struct_ident.span(), args.status)?;

    let struct_fields = match &args.data {
        Data::Struct(s) => s,
        _ => {
            return Err(syn::Error::new_spanned(
                struct_ident,
                "Response can only be applied to an tuple struct.",
            )
            .into())
        }
    };

    let mut into_response_arm_vec = Vec::new();
    let mut meta_response_vec = Vec::new();
    let mut schema_type_vec = Vec::new();

    let struct_description = get_description(&args.attrs)?;
    let struct_description = optional_literal(&struct_description);
    let (value_fields, header_fields) = parse_fields(struct_fields)?;

    let mut matched_header_idents = Vec::new();
    let mut insert_response_with_headers = Vec::new();
    let mut meta_headers = Vec::new();

    // headers
    for (idx, header_field) in header_fields.iter().enumerate() {
        let header_ident = quote::format_ident!("__p{}", idx);
        let header_name = header_field.header.as_ref().unwrap().to_uppercase();
        let header_ty = &header_field.ty;
        let header_desc = optional_literal_string(&get_description(&header_field.attrs)?);
        let deprecated = header_field.deprecated;

        insert_response_with_headers.push(quote! {
            if let Some(header) = ::poem_openapi::types::ToHeader::to_header(&#header_ident) {
                resp.headers_mut().insert(#header_name, header);
            }
        });
        matched_header_idents.push(header_ident);
        meta_headers.push(quote! {
            ::poem_openapi::registry::MetaHeader {
                name: ::std::string::ToString::to_string(#header_name),
                description: #header_desc,
                required: <#header_ty as ::poem_openapi::types::Type>::IS_REQUIRED,
                deprecated: #deprecated,
                schema: <#header_ty as ::poem_openapi::types::Type>::schema_ref(),
            }
        });
    }

    // extra headers
    for extra_header in args.headers.iter() {
        let name = extra_header.name.to_uppercase();
        let description = optional_literal_string(&extra_header.description);
        let ty = match syn::parse_str::<Type>(&extra_header.ty) {
            Ok(ty) => ty,
            Err(_) => return Err(syn::Error::new(extra_header.ty.span(), "Invalid type").into()),
        };
        let deprecated = extra_header.deprecated;

        meta_headers.push(quote! {
            ::poem_openapi::registry::MetaHeader {
                name: ::std::string::ToString::to_string(#name),
                description: #description,
                required: <#ty as ::poem_openapi::types::Type>::IS_REQUIRED,
                deprecated: #deprecated,
                schema: <#ty as ::poem_openapi::types::Type>::schema_ref(),
            }
        });
    }

    match value_fields.len() {
        1 => {
            // Field(media)
            let media_ty = &value_fields[0].ty;
            let (update_response_content_type, update_meta_content_type) =
                update_content_type(args.content_type.as_deref());
            into_response_arm_vec.push(quote! {
                #struct_ident(media, #(#matched_header_idents),*) => {
                    let mut resp = ::poem_openapi::__private::poem::IntoResponse::into_response(media);
                    resp.set_status(::poem_openapi::__private::poem::http::StatusCode::from_u16(#status).unwrap());
                    #(#insert_response_with_headers)*
                    #update_response_content_type
                    resp
                }
            });
            meta_response_vec.push(quote! {
                ::poem_openapi::registry::MetaResponse {
                    description: #struct_description.unwrap_or_default(),
                    status: ::std::option::Option::Some(#status),
                    content: {
                        let mut content = <#media_ty as ::poem_openapi::ResponseContent>::media_types();
                        #update_meta_content_type
                        content
                    },
                    headers: ::std::vec![#(#meta_headers),*],
                }
            });
            schema_type_vec.push(media_ty);
        }
        0 => {
            // Field
            let field = if !header_fields.is_empty() {
                quote!(#struct_ident(#(#matched_header_idents),*))
            } else {
                quote!(#struct_ident)
            };
            into_response_arm_vec.push(quote! {
                #field => {
                    let status = ::poem_openapi::__private::poem::http::StatusCode::from_u16(#status).unwrap();
                    #[allow(unused_mut)]
                    let mut resp = ::poem_openapi::__private::poem::IntoResponse::into_response(status);
                    #(#insert_response_with_headers)*
                    resp
                }
            });
            meta_response_vec.push(quote! {
                ::poem_openapi::registry::MetaResponse {
                    description: #struct_description.unwrap_or_default(),
                    status: ::std::option::Option::Some(#status),
                    content: ::std::vec![],
                    headers: ::std::vec![#(#meta_headers),*],
                }
            });
        }
        _ => {
            return Err(
                syn::Error::new_spanned(struct_ident, "Incorrect response definition.").into(),
            )
        }
    }

    let bad_request_handler_const = match &args.bad_request_handler {
        Some(_) => quote!(
            const BAD_REQUEST_HANDLER: bool = true;
        ),
        None => quote!(
            const BAD_REQUEST_HANDLER: bool = false;
        ),
    };
    let bad_request_handler = args.bad_request_handler.as_ref().map(|path| {
        quote! {
            fn from_parse_request_error(err: ::poem_openapi::__private::poem::Error) -> Self {
                #path(err)
            }
        }
    });

    let expanded = {
        quote! {
            impl #impl_generics ::poem_openapi::__private::poem::IntoResponse for #struct_ident #ty_generics #where_clause {
                fn into_response(self) -> ::poem_openapi::__private::poem::Response {
                    match self {
                        #(#into_response_arm_vec)*
                    }
                }
            }

            impl #impl_generics ::poem_openapi::ApiResponse for #struct_ident #ty_generics #where_clause {
                #bad_request_handler_const

                fn meta() -> ::poem_openapi::registry::MetaResponses {
                    ::poem_openapi::registry::MetaResponses {
                        responses: ::std::vec![#(#meta_response_vec),*]
                    }
                }

                fn register(registry: &mut ::poem_openapi::registry::Registry) {
                    #(<#schema_type_vec as ::poem_openapi::ResponseContent>::register(registry);)*
                }

                #bad_request_handler
            }

            impl #impl_generics ::std::convert::From<#struct_ident #ty_generics> for ::poem_openapi::__private::poem::Error #where_clause {
                fn from(resp: #struct_ident #ty_generics) -> ::poem_openapi::__private::poem::Error {
                    use ::poem_openapi::__private::poem::IntoResponse;
                    ::poem_openapi::__private::poem::Error::from_response(resp.into_response())
                }
            }
        }
    };

    Ok(expanded)
}

fn parse_fields(
    fields: &Fields<ResponseField>,
) -> syn::Result<(Vec<&ResponseField>, Vec<&ResponseField>)> {
    let mut value_fields = Vec::new();
    let mut header_fields = Vec::new();

    for field in &fields.fields {
        if field.header.is_some() {
            header_fields.push(field);
        } else {
            value_fields.push(field);
        }
    }

    Ok((value_fields, header_fields))
}

fn get_description(attrs: &[Attribute]) -> syn::Result<Option<String>> {
    let mut full_docs = String::new();
    for attr in attrs {
        if attr.path.is_ident("doc") {
            if let Meta::NameValue(nv) = attr.parse_meta()? {
                if let Lit::Str(doc) = nv.lit {
                    let doc = doc.value();
                    let doc_str = doc.trim();
                    if !full_docs.is_empty() {
                        full_docs += "\n";
                    }
                    full_docs += doc_str;
                }
            }
        }
    }
    Ok(if full_docs.is_empty() {
        None
    } else {
        Some(full_docs)
    })
}

fn optional_literal(s: &Option<impl AsRef<str>>) -> TokenStream {
    match s {
        Some(s) => {
            let s = s.as_ref();
            quote!(::std::option::Option::Some(#s))
        }
        None => quote!(::std::option::Option::None),
    }
}

fn optional_literal_string(s: &Option<impl AsRef<str>>) -> TokenStream {
    match s {
        Some(s) => {
            let s = s.as_ref();
            quote!(::std::option::Option::Some(::std::string::ToString::to_string(#s)))
        }
        None => quote!(::std::option::Option::None),
    }
}

fn update_content_type(content_type: Option<&str>) -> (TokenStream, TokenStream) {
    let update_response_content_type = match content_type {
        Some(content_type) => {
            quote! {
                resp.headers_mut().insert(::poem_openapi::__private::poem::http::header::CONTENT_TYPE,
                    ::poem_openapi::__private::poem::http::HeaderValue::from_static(#content_type));
            }
        }
        None => quote!(),
    };

    let update_meta_content_type = match content_type {
        Some(content_type) => quote! {
            if let Some(media_type) = content.get_mut(0) {
                media_type.content_type = #content_type;
            }
        },
        None => quote!(),
    };

    (update_response_content_type, update_meta_content_type)
}

fn get_status(span: Span, status: u16) -> GeneratorResult<TokenStream> {
    if !(100..1000).contains(&status) {
        return Err(syn::Error::new(
            span,
            "Invalid status code, it must be greater or equal to 100 and less than 1000.",
        )
        .into());
    }
    Ok(quote!(#status))
}
