# poem-extensions

[![Crates.io version](https://img.shields.io/crates/v/poem-extensions.svg?style=flat-square)](https://crates.io/crates/poem-extensions)

Add some extensions to Poem web framework.

## `UniOpenApi`, `api`

`UniOpenApi` unifies multiple `struct`s that implement [`OpenApi`](https://docs.rs/poem-openapi/latest/poem_openapi/attr.OpenApi.html) into one `struct`. Because using the [`OpenApiService::new()`](https://docs.rs/poem-openapi/latest/poem_openapi/struct.OpenApiService.html#method.new) method can only convert a tuple with at most 16 elements into an [`Endpoint`](https://docs.rs/poem/latest/poem/endpoint/trait.Endpoint.html#), `UniOpenApi` is available to facilitate developers to define an unlimited number of `OpenApi` implementations. `api` is a simplified version of `UniOpenApi`, combining declaration and invocation into one.

### Example

#### before

```rust
use poem_openapi::{OpenApi, OpenApiService};

struct Api1;

#[OpenApi]
impl Api1 {}

struct Api2;

#[OpenApi]
impl Api2 {}

struct Api3;

#[OpenApi]
impl Api3 {}

/// only put a maximum of 16 OpenApi struct
let api = (Api1, Api2, Api3);

let api_service = OpenApiService::new(api, "Combined APIs", "1.0")
        .server("http://localhost:3000/api");
```

#### after

```rust
use poem_extensions::{api, UniOpenApi};
use poem_openapi::{OpenApi, OpenApiService};

struct Api1;

#[OpenApi]
impl Api1 {}

struct Api2;

#[OpenApi]
impl Api2 {}

struct Api3;

#[OpenApi]
impl Api3 {}

/// struct mode, support generics
#[derive(UniOpenApi)]
struct Union(Api1, Api2, Api3);

let api = Union(Api1, Api2, Api3);

/// ... or tuple mode, not support generics
let api = api!(Api1, Api2, Api3);

let api_service = OpenApiService::new(api, "Combined APIs", "1.0")
        .server("http://localhost:3000/api");
```

## `OneResponse`, `UniResponse`, `response`

The response type defined by [ApiResponse](https://docs.rs/poem-openapi/latest/poem_openapi/derive.ApiResponse.html) has too much control granularity and is less reusable. Either one request defines one response, which is too much code, or it defines a response that contains all possible responses, which can obscure the really important ones.

Because of such shortcomings, 3 helpers are provided in this repository.

- `OneResponse` is a simplification of `ApiResponse`, where only one response type corresponding to one status code can be defined.
- `UniResponse` is an `enum` with 60 generic type slots corresponding to 60 response status codes.
- `response` is a functional macro for insert response type that defined by `OneResponse` into `UniResponse` type slots.

### Example

#### before

```rust
use poem_openapi::{param::Query, ApiResponse, OpenApi};

#[derive(ApiResponse)]
enum FirstResp {
    #[oai(status = 200)]
    Ok,
    #[oai(status = 400)]
    BadRequest,
}

#[derive(ApiResponse)]
enum SecondResp {
    #[oai(status = 200)]
    Ok,
    #[oai(status = 400)]
    BadRequest,
    #[oai(status = 404)]
    NotFound,
}

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/first", method = "get")]
    async fn first(&self, name: Query<Option<u64>>) -> FirstResp {
        match name.0 {
            Some(_) => FirstResp::Ok,
            None => FirstResp::BadRequest,
        }
    }

    #[oai(path = "/second", method = "get")]
    async fn second(&self, name: Query<Option<u64>>) -> SecondResp {
        match name.0 {
            Some(a) if a > 100 => SecondResp::NotFound,
            Some(_) => SecondResp::Ok,
            None => SecondResp::BadRequest,
        }
    }
}
```

#### after

```rust
use poem::IntoResponse;
use poem_extensions::{
    response, OneResponse,
    UniResponse::{T200, T400, T404},
};
use poem_openapi::{
    param::Query,
    payload::{Payload, PlainText},
    OpenApi,
};

#[derive(OneResponse)]
#[oai(status = 400)]
struct BadRequest<T: IntoResponse + Payload>(T);

#[derive(OneResponse)]
#[oai(status = 404)]
struct NotFound;

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/first", method = "get")]
    async fn first(
        &self,
        name: Query<Option<u64>>,
    ) -> response! {
           200: PlainText<String>,
           400: BadRequest<PlainText<String>>,
       } {
        match name.0 {
            Some(a) => T200(PlainText(format!("{}", a))),
            None => T400(BadRequest(PlainText("name is required".to_string()))),
        }
    }

    #[oai(path = "/second", method = "get")]
    async fn second(
        &self,
        name: Query<Option<u64>>,
    ) -> response! {
           200: (),
           400: BadRequest<PlainText<String>>,
           404: NotFound,
       } {
        match name.0 {
            Some(a) if a > 100 => T404(NotFound),
            Some(_) => T200(()),
            None => T400(BadRequest(PlainText("name is required".to_string()))),
        }
    }
}
```

## Contributing

Thanks for your help improving the project! We are so happy to have you!
