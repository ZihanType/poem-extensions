use poem::{http::Method, test::TestClient};
use poem_extensions::{api, UniOpenApi};
use poem_openapi::{payload::Json, registry::MetaApi, OpenApi, OpenApiService};

#[tokio::test]
async fn path_and_method() {
    struct A;

    #[OpenApi]
    impl A {
        #[oai(path = "/helloA", method = "get")]
        async fn hello(&self) {}
    }

    struct B;

    #[OpenApi]
    impl B {
        #[oai(path = "/helloB", method = "post")]
        async fn hello(&self) {}
    }

    #[derive(UniOpenApi)]
    struct Uni(A, B);

    let mut metas: Vec<MetaApi> = Uni::meta();
    assert_eq!(metas.len(), 2);

    let meta_a: MetaApi = metas.remove(0);
    assert_eq!(meta_a.paths[0].path, "/helloA");
    assert_eq!(meta_a.paths[0].operations[0].method, Method::GET);

    let meta_b: MetaApi = metas.remove(0);
    assert_eq!(meta_b.paths[0].path, "/helloB");
    assert_eq!(meta_b.paths[0].operations[0].method, Method::POST);

    let ep = OpenApiService::new(Uni(A, B), "test", "1.0");
    let cli = TestClient::new(ep);
    cli.get("/helloA").send().await.assert_status_is_ok();
    cli.post("/helloB").send().await.assert_status_is_ok();
}

#[tokio::test]
async fn generic() {
    trait Service: Send + Sync + 'static {
        fn test(&self) -> String;
    }

    struct ServiceImplA;

    impl Service for ServiceImplA {
        fn test(&self) -> String {
            "ServiceImplA".to_string()
        }
    }

    struct ServiceImplB;

    impl Service for ServiceImplB {
        fn test(&self) -> String {
            "ServiceImplB".to_string()
        }
    }

    struct ControllerA<Svc> {
        svc: Svc,
    }

    #[OpenApi]
    impl<Svc: Service> ControllerA<Svc> {
        #[oai(path = "/aaa", method = "get")]
        async fn aaa(&self) -> Json<String> {
            Json(self.svc.test())
        }
    }

    struct ControllerB<Svc> {
        svc: Svc,
    }

    #[OpenApi]
    impl<Svc: Service> ControllerB<Svc> {
        #[oai(path = "/bbb", method = "post")]
        async fn bbb(&self) -> Json<String> {
            Json(self.svc.test())
        }
    }

    #[derive(UniOpenApi)]
    struct Uni<SvcA: Service, SvcB: Service>(ControllerA<SvcA>, ControllerB<SvcB>);

    let ep = OpenApiService::new(
        Uni(
            ControllerA { svc: ServiceImplA },
            ControllerB { svc: ServiceImplB },
        ),
        "test",
        "1.0",
    );
    let cli = TestClient::new(ep);

    let resp = cli.get("/aaa").send().await;
    resp.assert_status_is_ok();
    resp.assert_json("ServiceImplA").await;
    let resp = cli.post("/bbb").send().await;
    resp.assert_status_is_ok();
    resp.assert_json("ServiceImplB").await;
}

#[tokio::test]
async fn path_and_method_use_macro() {
    struct A;

    #[OpenApi]
    impl A {
        #[oai(path = "/helloA", method = "get")]
        async fn hello(&self) {}
    }

    struct B;

    #[OpenApi]
    impl B {
        #[oai(path = "/helloB", method = "post")]
        async fn hello(&self) {}
    }

    let ep = OpenApiService::new(api!(A, B), "test", "1.0");
    let cli = TestClient::new(ep);
    cli.get("/helloA").send().await.assert_status_is_ok();
    cli.post("/helloB").send().await.assert_status_is_ok();
}
