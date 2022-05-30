use poem::{Error, IntoResponse};
use poem_openapi::{
    registry::{MetaResponses, Registry},
    ApiResponse,
};

use crate::error_response::ErrorResponse;

pub(crate) enum EitherResponse<T> {
    Extern(T),
    Intern(ErrorResponse),
}

impl<T: ApiResponse> ApiResponse for EitherResponse<T> {
    const BAD_REQUEST_HANDLER: bool = true;

    fn meta() -> MetaResponses {
        let mut meta = ErrorResponse::meta();
        let intern = &mut meta.responses;

        intern.sort_by(|a, b| a.status.cmp(&b.status));

        T::meta().responses.into_iter().for_each(|ext| {
            if let Ok(idx) = intern.binary_search_by(|int| int.status.cmp(&ext.status)) {
                intern.remove(idx);
            }
            intern.push(ext);
            intern.sort_by(|a, b| a.status.cmp(&b.status));
        });

        meta
    }

    fn register(registry: &mut Registry) {
        ErrorResponse::register(registry);
        T::register(registry);
    }

    fn from_parse_request_error(err: Error) -> Self {
        if T::BAD_REQUEST_HANDLER {
            Self::Extern(T::from_parse_request_error(err))
        } else {
            Self::Intern(ErrorResponse::from_parse_request_error(err))
        }
    }
}

impl<T: IntoResponse> IntoResponse for EitherResponse<T> {
    fn into_response(self) -> poem::Response {
        match self {
            Self::Extern(e) => e.into_response(),
            Self::Intern(i) => i.into_response(),
        }
    }
}
