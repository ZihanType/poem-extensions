# poem-extensions

Add some extensions to Poem web framework.
In this repo:
  - poem-openapi-response
  - poem-openapi-api-derive

## poem-openapi-response

Uniform response for [poem-openapi](https://docs.rs/poem-openapi).

### Example

```rust
use poem::{listener::TcpListener, EndpointExt, Route, Server};
use poem_openapi::{payload::PlainText, OpenApi, OpenApiService};
use poem_openapi_response::{ErrorResponse, UniResponse};

struct Api;

#[OpenApi]
impl Api {
    /// Hello world
    #[oai(path = "/hello", method = "get")]
    async fn hello(&self) -> UniResponse<PlainText<&'static str>> {
        UniResponse::new(PlainText("Hello World"))
    }

    /// Not found
    #[oai(path = "/not_found", method = "get")]
    async fn not_found(&self) -> UniResponse {
        UniResponse::not_found(None)
    }
}

let api_service =
        OpenApiService::new(Api, "Hello World", "1.0").server("http://localhost:3000");
let ui = api_service.swagger_ui();
let app = Route::new()
    .nest("/", api_service)
    .nest("/docs", ui)
    .catch_all_error(|e| async move {
        ErrorResponse::from_poem_error(&e)
                .unwrap_or_else(|| ErrorResponse::InternalServerError(PlainText(e.to_string())))
    });
Server::new(TcpListener::bind("127.0.0.1:3000"))
    .run(app)
    .await
    .unwrap();
```

## poem-openapi-api-derive

`UniOpenApi` unifies multiple `struct`s that implement [`OpenApi`](https://docs.rs/poem-openapi/latest/poem_openapi/attr.OpenApi.html) into one `struct`. Because using the [`OpenApiService::new()`](https://docs.rs/poem-openapi/latest/poem_openapi/struct.OpenApiService.html#method.new) method can only convert a tuple with at most 16 elements into an [`Endpoint`](https://docs.rs/poem/latest/poem/endpoint/trait.Endpoint.html#), UniOpenApi is available to facilitate developers to define an unlimited number of `OpenApi` implementations.

### Example

```rust
use poem::{listener::TcpListener, Route, Server};
use poem_openapi::{payload::PlainText, OpenApi, OpenApiService};
use poem_openapi_api_derive::UniOpenApi;

struct A;

#[OpenApi]
impl A {
    #[oai(path = "/helloA", method = "get")]
    async fn hello(&self) -> PlainText<&'static str> {
        PlainText("Hello World A")
    }
}

struct B;

#[OpenApi]
impl B {
    #[oai(path = "/helloB", method = "get")]
    async fn hello(&self) -> PlainText<&'static str> {
        PlainText("Hello World B")
    }
}

#[derive(UniOpenApi)]
struct Uni(A, B);

let api_service =
    OpenApiService::new(Uni(A, B), "Hello World", "1.0").server("http://localhost:3000");
let ui = api_service.swagger_ui();
let app = Route::new().nest("/", api_service).nest("/docs", ui);
Server::new(TcpListener::bind("127.0.0.1:3000"))
    .run(app)
    .await
    .unwrap();
```