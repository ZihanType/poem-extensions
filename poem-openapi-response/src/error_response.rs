use poem::{
    error::{
        CorsError, GetDataError, MethodNotAllowedError, MissingJsonContentTypeError, NotFoundError,
        ParseCookieError, ParseFormError, ParseJsonError,
        ParseMultipartError as PoemParseMultipartError, ParsePathError, ParseQueryError,
        ParseTypedHeaderError, ReadBodyError, ResponseError, RouteError, SizedLimitError,
        StaticFileError, UpgradeError,
    },
    Error,
};
use poem_openapi::{
    error::{
        AuthorizationError, ContentTypeError,
        ParseMultipartError as PoemOpenApiParseMultipartError, ParseParamError,
        ParseRequestPayloadError,
    },
    payload::PlainText,
    ApiResponse,
};
use std::error::Error as StdError;

#[derive(ApiResponse)]
#[oai(bad_request_handler = "bad_request_handler")]
pub enum ErrorResponse {
    #[cfg(feature = "400")]
    #[oai(status = 400)]
    BadRequest(PlainText<String>),

    #[cfg(feature = "401")]
    #[oai(status = 401)]
    Unauthorized(PlainText<String>),

    #[cfg(feature = "403")]
    #[oai(status = 403)]
    Forbidden(PlainText<String>),

    #[cfg(feature = "404")]
    #[oai(status = 404)]
    NotFound(PlainText<String>),

    #[cfg(feature = "405")]
    #[oai(status = 405)]
    MethodNotAllowed(PlainText<String>),

    #[cfg(feature = "412")]
    #[oai(status = 412)]
    PreconditionFailed(PlainText<String>),

    #[cfg(feature = "413")]
    #[oai(status = 413)]
    PayloadTooLarge(PlainText<String>),

    #[cfg(feature = "415")]
    #[oai(status = 415)]
    UnsupportedMediaType(PlainText<String>),

    #[cfg(feature = "416")]
    #[oai(status = 416)]
    RangeNotSatisfiable(
        PlainText<String>,
        #[oai(header = "content-range")] Option<u64>,
    ),

    #[oai(status = 500)]
    InternalServerError(PlainText<String>),
}

macro_rules! return_from_response_error {
    (
        $err:expr;
        $status:expr;
        $(
            $(#[$attr:meta])*
            ($uint16:expr, $variant:ident);
        )*
    ) => {
        $(
            $(#[$attr])*
            if $status.as_u16() == $uint16 {
                return Some(ErrorResponse::$variant(PlainText($err.to_string())));
            }
        )*
    };
}

macro_rules! return_from_poem_error {
    (
        $err:expr;
        $(
            $err_type:ident;
        )*
    ) => {
        $(
            if let Some(e) = $err.downcast_ref::<$err_type>() {
                return ErrorResponse::from_response_error(e);
            }
        )*
    };
}

impl ErrorResponse {
    pub fn from_response_error<T>(err: &T) -> Option<Self>
    where
        T: ResponseError + StdError + Send + Sync + 'static,
    {
        let status = err.status();

        return_from_response_error! {
            err;
            status;

            #[cfg(feature = "400")]
            (400, BadRequest);

            #[cfg(feature = "401")]
            (401, Unauthorized);

            #[cfg(feature = "403")]
            (403, Forbidden);

            #[cfg(feature = "404")]
            (404, NotFound);

            #[cfg(feature = "405")]
            (405, MethodNotAllowed);

            #[cfg(feature = "412")]
            (412, PreconditionFailed);

            #[cfg(feature = "413")]
            (413, PayloadTooLarge);

            #[cfg(feature = "415")]
            (415, UnsupportedMediaType);

            (500, InternalServerError);
        }

        #[cfg(feature = "416")]
        if status.as_u16() == 416 {
            let response = err.as_response();
            let content_range = response.header("content-range");
            return Some(ErrorResponse::RangeNotSatisfiable(
                PlainText(err.to_string()),
                content_range.and_then(|a| a.parse::<u64>().ok()),
            ));
        }

        None
    }

    pub fn from_poem_error(err: &Error) -> Option<Self> {
        return_from_poem_error! {
            err;
            ParsePathError;
            NotFoundError;
            MethodNotAllowedError;
            CorsError;
            ReadBodyError;
            ParseCookieError;
            GetDataError;
            ParseFormError;
            ParseJsonError;
            MissingJsonContentTypeError;
            ParseQueryError;
            PoemParseMultipartError;
            ParseTypedHeaderError;
            UpgradeError;
            StaticFileError;
            SizedLimitError;
            RouteError;
            ParseParamError;
            ParsePathError;
            ParseRequestPayloadError;
            PoemOpenApiParseMultipartError;
            ContentTypeError;
            AuthorizationError;
        }

        None
    }
}

fn bad_request_handler(err: Error) -> ErrorResponse {
    ErrorResponse::from_poem_error(&err)
        .unwrap_or_else(|| ErrorResponse::InternalServerError(PlainText(err.to_string())))
}
