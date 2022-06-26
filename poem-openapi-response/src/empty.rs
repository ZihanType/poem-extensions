use poem::{Body, IntoResponse, Response};
use poem_openapi::{
    registry::{MetaResponses, Registry},
    ApiResponse,
};

pub struct Empty;

impl IntoResponse for Empty {
    fn into_response(self) -> Response {
        Response::builder().body(Body::empty())
    }
}

impl ApiResponse for Empty {
    fn meta() -> MetaResponses {
        MetaResponses {
            responses: Vec::new(),
        }
    }

    fn register(_registry: &mut Registry) {}
}
