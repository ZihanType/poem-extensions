use poem::{Error, IntoResponse};
use poem_openapi::{
    payload::PlainText,
    registry::{MetaResponses, Registry},
    ApiResponse,
};

use crate::{either_response::EitherResponse, ErrorResponse};

pub struct UniResponse<T = ()> {
    either: EitherResponse<T>,
}

impl<T: ApiResponse> UniResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            either: EitherResponse::Extern(data),
        }
    }
}

macro_rules! define_associated_function {
    (
        $(
            $(#[$attr:meta])*
            ($name:ident, $variant:ident, $phrase:expr);
        )*
    ) => {
        $(
            $(#[$attr])*
            pub fn $name<M: Into<Option<String>>>(msg: M) -> UniResponse {
                UniResponse {
                    either: EitherResponse::Intern(ErrorResponse::$variant(PlainText(
                        msg.into().unwrap_or_else(|| $phrase.to_string()),
                    ))),
                }
            }
        )*
    };
}

impl UniResponse {
    define_associated_function! {
        #[cfg(feature = "400")]
        (bad_request, BadRequest, "Bad Request");

        #[cfg(feature = "401")]
        (unauthorized, Unauthorized, "Unauthorized");

        #[cfg(feature = "403")]
        (forbidden, Forbidden, "Forbidden");

        #[cfg(feature = "404")]
        (not_found, NotFound, "Not Found");

        #[cfg(feature = "405")]
        (method_not_allowed, MethodNotAllowed, "Method Not Allowed");

        #[cfg(feature = "412")]
        (precondition_failed, PreconditionFailed, "Precondition Failed");

        #[cfg(feature = "413")]
        (payload_too_large, PayloadTooLarge, "Payload Too Large");

        #[cfg(feature = "415")]
        (unsupported_media_type, UnsupportedMediaType, "Unsupported Media Type");

        (error, InternalServerError, "Internal Server Error");
    }

    #[cfg(feature = "416")]
    pub fn range_not_satisfiable<M: Into<Option<String>>, R: Into<Option<u64>>>(
        msg: M,
        content_range: R,
    ) -> UniResponse {
        UniResponse {
            either: EitherResponse::Intern(ErrorResponse::RangeNotSatisfiable(
                PlainText(
                    msg.into()
                        .unwrap_or_else(|| "Range Not Satisfiable".to_string()),
                ),
                content_range.into(),
            )),
        }
    }
}

impl<T: ApiResponse> ApiResponse for UniResponse<T> {
    const BAD_REQUEST_HANDLER: bool = EitherResponse::<T>::BAD_REQUEST_HANDLER;

    fn meta() -> MetaResponses {
        EitherResponse::<T>::meta()
    }

    fn register(registry: &mut Registry) {
        EitherResponse::<T>::register(registry);
    }

    fn from_parse_request_error(err: Error) -> Self {
        Self {
            either: EitherResponse::<T>::from_parse_request_error(err),
        }
    }
}

impl<T: IntoResponse> IntoResponse for UniResponse<T> {
    fn into_response(self) -> poem::Response {
        self.either.into_response()
    }
}
