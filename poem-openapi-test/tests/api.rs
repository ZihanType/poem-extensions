use poem::{http::Method, test::TestClient};
use poem_openapi::{payload::PlainText, OpenApi, OpenApiService};
use poem_openapi_macro::UniOpenApi;

#[tokio::test]
async fn path_and_method() {
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
        #[oai(path = "/helloB", method = "post")]
        async fn hello(&self) -> PlainText<&'static str> {
            PlainText("Hello World B")
        }
    }

    #[derive(UniOpenApi)]
    struct Uni(A, B);

    let mut metas = Uni::meta();
    assert_eq!(metas.len(), 2);

    let meta_a = metas.remove(0);
    assert_eq!(meta_a.paths[0].path, "/helloA");
    assert_eq!(meta_a.paths[0].operations[0].method, Method::GET);

    let meta_b = metas.remove(0);
    assert_eq!(meta_b.paths[0].path, "/helloB");
    assert_eq!(meta_b.paths[0].operations[0].method, Method::POST);

    let ep = OpenApiService::new(Uni(A, B), "test", "1.0");
    let cli = TestClient::new(ep);
    cli.get("/helloA").send().await.assert_status_is_ok();
    cli.post("/helloB").send().await.assert_status_is_ok();
}
