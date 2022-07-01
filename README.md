# poem-extensions

Add some extensions to Poem web framework.

## `UniOpenApi`

`UniOpenApi` unifies multiple `struct`s that implement [`OpenApi`](https://docs.rs/poem-openapi/latest/poem_openapi/attr.OpenApi.html) into one `struct`. Because using the [`OpenApiService::new()`](https://docs.rs/poem-openapi/latest/poem_openapi/struct.OpenApiService.html#method.new) method can only convert a tuple with at most 16 elements into an [`Endpoint`](https://docs.rs/poem/latest/poem/endpoint/trait.Endpoint.html#), UniOpenApi is available to facilitate developers to define an unlimited number of `OpenApi` implementations.

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
use poem_openapi::{OpenApi, OpenApiService};
use poem_openapi_macro::UniOpenApi;

struct Api1;

#[OpenApi]
impl Api1 {}

struct Api2;

#[OpenApi]
impl Api2 {}

struct Api3;

#[OpenApi]
impl Api3 {}

/// unlimit
#[derive(UniOpenApi)]
struct Union(Api1, Api2, Api3);

/// unlimit
let api = Union(Api1, Api2, Api3);

let api_service = OpenApiService::new(api, "Combined APIs", "1.0")
        .server("http://localhost:3000/api");
```

## `response`, `OneResponse`, `UniResponse`

The response type defined by [ApiResponse](https://docs.rs/poem-openapi/latest/poem_openapi/derive.ApiResponse.html) has too much control granularity and is less reusable. Either one request defines one response, which is too much code, or it defines a response that contains all possible responses, which can obscure the really important ones.

Because of such shortcomings, 3 helpers are provided in this repository.

- `UniResponse` is an `enum` with 11 generic type slots corresponding to 11 response status codes, and the default type is not displayed in Swagger if no generic type is inserted into the corresponding status code.
- `OneResponse` is a simplification of `ApiResponse`, where only one response type corresponding to one status code can be defined.
- `response` is a functional macro for insert really response type into `UniResponse` type slots at the function return type.

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
use poem_openapi::{
    param::Query,
    payload::{Payload, PlainText},
    OpenApi,
};
use poem_openapi_response::{response, OneResponse, UniResponse::*};

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
