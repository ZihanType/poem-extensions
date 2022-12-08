use poem::{http::StatusCode, test::TestClient};
use poem_extensions::{response, OneResponse, UniResponse};
use poem_openapi::{
    payload::Json,
    registry::{MetaApi, MetaResponse, MetaResponses},
    ApiResponse, OpenApi, OpenApiService,
};

#[test]
fn meta() {
    /// Ok
    #[derive(OneResponse)]
    #[oai(status = 200)]
    struct Ok;

    #[derive(OneResponse)]
    #[oai(status = 201)]
    struct Created;

    /// A
    /// B
    ///
    /// C
    #[derive(OneResponse)]
    #[oai(status = 400)]
    struct BadRequest;

    assert_eq!(
        <response! {
            200: Ok,
        }>::meta(),
        MetaResponses {
            responses: vec![MetaResponse {
                description: "Ok",
                status: Some(200),
                content: vec![],
                headers: vec![]
            }],
        },
    );

    assert_eq!(
        <response! {
            201: Created,
            400: BadRequest,
        }>::meta(),
        MetaResponses {
            responses: vec![
                MetaResponse {
                    description: "",
                    status: Some(201),
                    content: vec![],
                    headers: vec![]
                },
                MetaResponse {
                    description: "A\nB\n\nC",
                    status: Some(400),
                    content: vec![],
                    headers: vec![]
                }
            ],
        },
    );
}

#[tokio::test]
async fn multiple_responses() {
    #[derive(OneResponse, Debug)]
    #[oai(status = 200)]
    struct Ok;

    #[derive(OneResponse, Debug)]
    #[oai(status = 201)]
    struct Created;

    #[derive(OneResponse, Debug)]
    #[oai(status = 502)]
    struct BadGateway;

    struct Api;

    #[OpenApi]
    impl Api {
        #[oai(path = "/number", method = "post")]
        async fn number(
            &self,
            num: Json<u16>,
        ) -> response! {
               200: Ok,
               201: Created,
               502: BadGateway,
           } {
            let Json(num) = num;
            if num == 200 {
                UniResponse::T200(Ok)
            } else if num == 201 {
                UniResponse::T201(Created)
            } else {
                UniResponse::T502(BadGateway)
            }
        }
    }

    let ep = OpenApiService::new(Api, "test", "1.0");
    let cli = TestClient::new(ep);

    cli.post("/number")
        .content_type("application/json")
        .body("200")
        .send()
        .await
        .assert_status(StatusCode::OK);

    cli.post("/number")
        .content_type("application/json")
        .body("201")
        .send()
        .await
        .assert_status(StatusCode::CREATED);

    cli.post("/number")
        .content_type("application/json")
        .body("502")
        .send()
        .await
        .assert_status(StatusCode::BAD_GATEWAY);

    let meta: MetaApi = Api::meta().remove(0);
    assert_eq!(meta.paths[0].path, "/number");
    let responses = &meta.paths[0].operations[0].responses.responses;
    assert_eq!(responses[0].status, Some(200));
    assert_eq!(responses[1].status, Some(201));
    assert_eq!(responses[2].status, Some(502));
}
