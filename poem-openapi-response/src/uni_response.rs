use crate::Empty;
use poem::{IntoResponse, Response};
use poem_openapi::{
    registry::{MetaResponse, MetaResponses, Registry},
    ApiResponse,
};
use std::any::type_name;

pub enum UniResponse<
    T200 = Empty,
    T400 = Empty,
    T401 = Empty,
    T403 = Empty,
    T404 = Empty,
    T405 = Empty,
    T412 = Empty,
    T413 = Empty,
    T415 = Empty,
    T416 = Empty,
    T500 = Empty,
> {
    T200(T200),
    T400(T400),
    T401(T401),
    T403(T403),
    T404(T404),
    T405(T405),
    T412(T412),
    T413(T413),
    T415(T415),
    T416(T416),
    T500(T500),
}

impl<T200, T400, T401, T403, T404, T405, T412, T413, T415, T416, T500> ApiResponse
    for UniResponse<T200, T400, T401, T403, T404, T405, T412, T413, T415, T416, T500>
where
    T200: ApiResponse,
    T400: ApiResponse,
    T401: ApiResponse,
    T403: ApiResponse,
    T404: ApiResponse,
    T405: ApiResponse,
    T412: ApiResponse,
    T413: ApiResponse,
    T415: ApiResponse,
    T416: ApiResponse,
    T500: ApiResponse,
{
    const BAD_REQUEST_HANDLER: bool = false;

    fn meta() -> MetaResponses {
        let mut responses = vec![];

        meta_helper::<T200>(&mut responses, 200);
        meta_helper::<T400>(&mut responses, 400);
        meta_helper::<T401>(&mut responses, 401);
        meta_helper::<T403>(&mut responses, 403);
        meta_helper::<T404>(&mut responses, 404);
        meta_helper::<T405>(&mut responses, 405);
        meta_helper::<T412>(&mut responses, 412);
        meta_helper::<T413>(&mut responses, 413);
        meta_helper::<T415>(&mut responses, 415);
        meta_helper::<T416>(&mut responses, 416);
        meta_helper::<T500>(&mut responses, 500);

        MetaResponses { responses }
    }

    fn register(registry: &mut Registry) {
        T200::register(registry);
        T400::register(registry);
        T401::register(registry);
        T403::register(registry);
        T404::register(registry);
        T405::register(registry);
        T412::register(registry);
        T413::register(registry);
        T415::register(registry);
        T416::register(registry);
        T500::register(registry);
    }
}

fn meta_helper<T: ApiResponse>(outer_responses: &mut Vec<MetaResponse>, outer_status: u16) {
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
                type_name::<T>(),
                outer_status
            );
        }
    } else {
        panic!(
            "status code of the first response in {} responses is none",
            type_name::<T>()
        );
    }
}

impl<T200, T400, T401, T403, T404, T405, T412, T413, T415, T416, T500> IntoResponse
    for UniResponse<T200, T400, T401, T403, T404, T405, T412, T413, T415, T416, T500>
where
    T200: IntoResponse,
    T400: IntoResponse,
    T401: IntoResponse,
    T403: IntoResponse,
    T404: IntoResponse,
    T405: IntoResponse,
    T412: IntoResponse,
    T413: IntoResponse,
    T415: IntoResponse,
    T416: IntoResponse,
    T500: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            UniResponse::T200(t) => t.into_response(),
            UniResponse::T400(t) => t.into_response(),
            UniResponse::T401(t) => t.into_response(),
            UniResponse::T403(t) => t.into_response(),
            UniResponse::T404(t) => t.into_response(),
            UniResponse::T405(t) => t.into_response(),
            UniResponse::T412(t) => t.into_response(),
            UniResponse::T413(t) => t.into_response(),
            UniResponse::T415(t) => t.into_response(),
            UniResponse::T416(t) => t.into_response(),
            UniResponse::T500(t) => t.into_response(),
        }
    }
}
