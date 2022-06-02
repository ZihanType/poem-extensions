//! Uniform response for [poem-openapi](https://docs.rs/poem-openapi).
//!
//! The `poem-openapi-response` is a poem-openapi extension crate, in order to unify the `poem-openapi`
//! response types.
//!
//! In this crate:
//! - [`ErrorResponse`] includes all the types defined in the Poem framework that implement [`ResponseError`](https://docs.rs/poem/latest/poem/error/trait.ResponseError.html),
//!   facilitating the generation of Responses from errors already defined by Poem.
//! - [`UniResponse<T>`] is shaped like [`Result<T, ErrorResponse>`], in general T means a response with status code 200,
//!   but it can also be any type that implements [`ApiResponse`](https://docs.rs/poem-openapi/latest/poem_openapi/trait.ApiResponse.html).
//!   Please note that, If the status codes defined in `T` and `ErrorResponse` are the same, the definition in `T` will override the definition in `ErrorResponse`. This is useful when
//!   you want to define a custom error response.
//!
//! ## Quickstart
//!
//! Cargo.toml
//!
//! ```toml
//! [package]
//! name = "helloworld"
//! version = "0.1.0"
//! edition = "2021"
//!
//! [dependencies]
//! poem = "1"
//! poem-openapi = { version = "2", features = ["swagger-ui"] }
//! poem-openapi-response = "0.2.0"
//! tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
//! ```
//!
//! main.rs
//!
//! ```no_run
//! use poem::{listener::TcpListener, EndpointExt, Route, Server};
//! use poem_openapi::{payload::PlainText, OpenApi, OpenApiService};
//! use poem_openapi_response::{ErrorResponse, UniResponse};
//!
//! struct Api;
//!
//! #[OpenApi]
//! impl Api {
//!     /// Hello world
//!     #[oai(path = "/hello", method = "get")]
//!     async fn hello(&self) -> UniResponse<PlainText<&'static str>> {
//!         UniResponse::new(PlainText("Hello World"))
//!     }
//!
//!     /// Not found
//!     #[oai(path = "/not_found", method = "get")]
//!     async fn not_found(&self) -> UniResponse {
//!         UniResponse::not_found(None)
//!     }
//! }
//!
//! let api_service =
//!     OpenApiService::new(Api, "Hello World", "1.0").server("http://localhost:3000");
//! let ui = api_service.swagger_ui();
//! let app = Route::new()
//!     .nest("/", api_service)
//!     .nest("/docs", ui)
//!     .catch_all_error(|e| async move {
//!         ErrorResponse::from_poem_error(e)
//!             .unwrap_or_else(|| ErrorResponse::InternalServerError(PlainText(e.to_string())))
//!     });
//!
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! Server::new(TcpListener::bind("127.0.0.1:3000"))
//!     .run(app)
//!     .await;
//! # });
//! ```

mod either_response;
mod error_response;
mod uni_response;

pub use error_response::ErrorResponse;
pub use uni_response::UniResponse;
