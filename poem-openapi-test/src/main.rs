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

#[tokio::main]
async fn main() {
    let api_service =
        OpenApiService::new(Uni(A, B), "Hello World", "1.0").server("http://localhost:3000");
    let ui = api_service.swagger_ui();
    let app = Route::new().nest("/", api_service).nest("/docs", ui);
    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await
        .unwrap();
}
