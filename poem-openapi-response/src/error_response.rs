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
    #[oai(status = 400)]
    BadRequest(PlainText<String>),
    #[oai(status = 401)]
    Unauthorized(PlainText<String>),
    #[oai(status = 403)]
    Forbidden(PlainText<String>),
    #[oai(status = 404)]
    NotFound(PlainText<String>),
    #[oai(status = 405)]
    MethodNotAllowed(PlainText<String>),
    #[oai(status = 412)]
    PreconditionFailed(PlainText<String>),
    #[oai(status = 413)]
    PayloadTooLarge(PlainText<String>),
    #[oai(status = 415)]
    UnsupportedMediaType(PlainText<String>),
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
            ($uint16:expr, $variant:ident);
        )*
    ) => {
        $(
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
            (400, BadRequest);
            (401, Unauthorized);
            (403, Forbidden);
            (404, NotFound);
            (405, MethodNotAllowed);
            (412, PreconditionFailed);
            (413, PayloadTooLarge);
            (415, UnsupportedMediaType);
            (500, InternalServerError);
        }

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
