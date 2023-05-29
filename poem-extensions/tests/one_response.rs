use std::fmt::{self, Display};

use poem::{
    http::{HeaderValue, StatusCode},
    test::TestClient,
    Error, IntoResponse,
};
use poem_extensions::OneResponse;
use poem_openapi::{
    payload::{Binary, Json, Payload},
    registry::{
        MetaApi, MetaMediaType, MetaResponse, MetaResponses, MetaSchema, MetaSchemaRef, Registry,
    },
    types::{ToJSON, Type},
    ApiResponse, Object, OpenApi, OpenApiService,
};
use serde_json::Value;

#[derive(Debug, Object)]
struct BadRequestResult {
    error_code: i32,
    message: String,
}

/// Ok
#[derive(OneResponse)]
#[oai(status = 200)]
struct Ok;

/// A
/// B
///
/// C
#[derive(OneResponse)]
#[oai(status = 400)]
struct BadRequest(Json<BadRequestResult>);

#[test]
fn meta() {
    assert_eq!(
        Ok::meta(),
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
        BadRequest::meta(),
        MetaResponses {
            responses: vec![MetaResponse {
                description: "A\nB\n\nC",
                status: Some(400),
                content: vec![MetaMediaType {
                    content_type: "application/json; charset=utf-8",
                    schema: MetaSchemaRef::Reference("BadRequestResult".to_string())
                }],
                headers: vec![]
            }],
        },
    );
}

#[tokio::test]
async fn into_response() {
    let resp = Ok.into_response();
    assert_eq!(resp.status(), StatusCode::OK);

    let mut resp = BadRequest(Json(BadRequestResult {
        error_code: 123,
        message: "abc".to_string(),
    }))
    .into_response();
    assert_eq!(
        serde_json::from_slice::<Value>(&resp.take_body().into_bytes().await.unwrap()).unwrap(),
        serde_json::json!({
            "error_code": 123,
            "message": "abc",
        })
    );
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn headers() {
    #[derive(OneResponse)]
    #[oai(status = 200)]
    struct A;

    #[derive(OneResponse)]
    #[oai(status = 200)]
    struct B(
        /// header1
        #[oai(header = "MY-HEADER1")]
        i32,
        #[oai(header = "MY-HEADER2")] Option<String>,
    );

    #[derive(OneResponse)]
    #[oai(status = 400)]
    struct C(
        Json<BadRequestResult>,
        #[oai(header = "MY-HEADER1")] i32,
        #[oai(header = "MY-HEADER2")] String,
    );

    let meta: MetaResponses = A::meta();
    assert_eq!(meta.responses[0].headers, &[]);

    let meta: MetaResponses = B::meta();
    let header1 = &meta.responses[0].headers[0];
    let header2 = &meta.responses[0].headers[1];

    assert_eq!(header1.name, "MY-HEADER1");
    assert_eq!(header1.description.as_deref(), Some("header1"));
    assert!(header1.required);
    assert_eq!(
        header1.schema,
        MetaSchemaRef::Inline(Box::new(MetaSchema::new_with_format("integer", "int32")))
    );

    assert_eq!(header2.name, "MY-HEADER2");
    assert_eq!(header2.description, None);
    assert!(!header2.required);
    assert_eq!(
        header2.schema,
        MetaSchemaRef::Inline(Box::new(MetaSchema::new("string")))
    );

    let resp = A.into_response();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = B(88, Some("abc".to_string())).into_response();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get("MY-HEADER1"),
        Some(&HeaderValue::from_static("88"))
    );
    assert_eq!(
        resp.headers().get("MY-HEADER2"),
        Some(&HeaderValue::from_static("abc"))
    );

    let resp = B(88, None).into_response();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get("MY-HEADER1"),
        Some(&HeaderValue::from_static("88"))
    );
    assert!(!resp.headers().contains_key("MY-HEADER2"));

    let mut resp = C(
        Json(BadRequestResult {
            error_code: 11,
            message: "hehe".to_string(),
        }),
        88,
        "abc".to_string(),
    )
    .into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        serde_json::from_slice::<Value>(&resp.take_body().into_bytes().await.unwrap()).unwrap(),
        serde_json::json!({
            "error_code": 11,
            "message": "hehe",
        })
    );
    assert_eq!(
        resp.headers().get("MY-HEADER1"),
        Some(&HeaderValue::from_static("88"))
    );
    assert_eq!(
        resp.headers().get("MY-HEADER2"),
        Some(&HeaderValue::from_static("abc"))
    );
}

#[tokio::test]
async fn bad_request_handler() {
    #[derive(OneResponse, Debug, Eq, PartialEq)]
    #[oai(bad_request_handler = "bad_request_handler")]
    #[oai(status = 400)]
    struct BadRequest;

    fn bad_request_handler(_: Error) -> BadRequest {
        BadRequest
    }

    assert_eq!(
        BadRequest::from_parse_request_error(Error::from_status(StatusCode::BAD_GATEWAY)),
        BadRequest
    );
}

#[tokio::test]
async fn generic() {
    #[derive(OneResponse)]
    #[oai(status = 200)]
    struct Ok<T: ToJSON>(Json<T>);

    assert_eq!(
        Ok::<String>::meta(),
        MetaResponses {
            responses: vec![MetaResponse {
                description: "",
                status: Some(200),
                content: vec![MetaMediaType {
                    content_type: "application/json; charset=utf-8",
                    schema: MetaSchemaRef::Inline(Box::new(MetaSchema::new("string")))
                }],
                headers: vec![]
            },],
        },
    );

    let mut resp = Ok(Json("success".to_string())).into_response();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        serde_json::from_slice::<Value>(&resp.take_body().into_bytes().await.unwrap()).unwrap(),
        serde_json::json!("success")
    );
}

#[tokio::test]
async fn item_content_type() {
    #[derive(OneResponse, Debug, Eq, PartialEq)]
    #[oai(status = 200, content_type = "application/json2")]
    struct A(Json<i32>);

    assert_eq!(
        A::meta(),
        MetaResponses {
            responses: vec![MetaResponse {
                description: "",
                status: Some(200),
                content: vec![MetaMediaType {
                    content_type: "application/json2",
                    schema: MetaSchemaRef::Inline(Box::new(MetaSchema::new_with_format(
                        "integer", "int32"
                    )))
                }],
                headers: vec![]
            },],
        },
    );

    let mut resp = A(Json(100)).into_response();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.content_type(), Some("application/json2"));
    assert_eq!(
        serde_json::from_slice::<Value>(&resp.take_body().into_bytes().await.unwrap()).unwrap(),
        serde_json::json!(100)
    );
}

#[tokio::test]
async fn header_deprecated() {
    #[derive(OneResponse, Debug, Eq, PartialEq)]
    #[oai(status = 200)]
    struct A(Json<i32>, #[oai(header = "A", deprecated = true)] String);

    let meta: MetaResponses = A::meta();
    assert!(meta.responses[0].headers[0].deprecated);
}

#[tokio::test]
async fn extra_headers_on_response() {
    #[derive(OneResponse, Debug, Eq, PartialEq)]
    #[oai(
        status = 200,
        header(name = "A1", ty = "String"),
        header(name = "a2", ty = "i32", description = "abc", deprecated = true)
    )]
    struct A(Json<i32>, #[oai(header = "A")] String);

    let meta: MetaResponses = A::meta();
    assert_eq!(meta.responses[0].headers.len(), 3);

    assert_eq!(meta.responses[0].headers[0].name, "A");
    assert!(!meta.responses[0].headers[0].deprecated);

    assert_eq!(meta.responses[0].headers[1].name, "A1");
    assert_eq!(meta.responses[0].headers[1].description, None);
    assert!(!meta.responses[0].headers[1].deprecated);
    assert_eq!(meta.responses[0].headers[1].schema, String::schema_ref());

    assert_eq!(meta.responses[0].headers[2].name, "A2");
    assert_eq!(
        meta.responses[0].headers[2].description.as_deref(),
        Some("abc")
    );
    assert!(meta.responses[0].headers[2].deprecated);
    assert_eq!(meta.responses[0].headers[2].schema, i32::schema_ref());
}

#[tokio::test]
async fn as_error() {
    #[derive(OneResponse, Debug)]
    #[oai(status = 200)]
    struct Ok;

    #[derive(OneResponse, Debug)]
    #[oai(status = 201)]
    struct Created;

    /// Bad gateway
    #[derive(OneResponse, Debug)]
    #[oai(status = 502)]
    struct BadGateway;

    struct Api;

    #[OpenApi]
    impl Api {
        #[oai(path = "/ok", method = "get")]
        async fn ok(&self) -> Result<Created, BadGateway> {
            Result::Ok(Created)
        }

        #[oai(path = "/err", method = "get")]
        async fn err(&self) -> Result<Ok, BadGateway> {
            Result::Err(BadGateway)
        }
    }

    let ep = OpenApiService::new(Api, "test", "1.0");
    let cli = TestClient::new(ep);

    let resp = cli.get("/ok").send().await;
    resp.assert_status(StatusCode::CREATED);

    let resp = cli.get("/err").send().await;
    resp.assert_status(StatusCode::BAD_GATEWAY);

    let meta: MetaApi = Api::meta().remove(0);
    assert_eq!(meta.paths[0].path, "/ok");
    let responses = &meta.paths[0].operations[0].responses.responses;
    assert_eq!(responses[0].status, Some(201));
    assert_eq!(responses[1].status, Some(502));

    assert_eq!(meta.paths[1].path, "/err");
    let responses = &meta.paths[1].operations[0].responses.responses;
    assert_eq!(responses[0].status, Some(200));
    assert_eq!(responses[1].status, Some(502));

    let err: Error = BadGateway.into();
    assert_eq!(err.to_string(), "Bad gateway");
}

#[tokio::test]
async fn display() {
    #[derive(Debug, OneResponse)]
    #[oai(status = 400, display)]
    struct InvalidValue(Json<i32>);

    impl Display for InvalidValue {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                InvalidValue(value) => write!(f, "invalid value: {}", value.0),
            }
        }
    }

    let err: Error = InvalidValue(Json(123)).into();
    assert_eq!(err.to_string(), "invalid value: 123");
}

#[tokio::test]
async fn actual_type() {
    #[derive(Debug, Object)]
    struct MyObj {
        value: i32,
    }

    #[derive(Debug, OneResponse)]
    #[oai(status = 200, actual_type = "Json<MyObj>")]
    struct Ok(Binary<Vec<u8>>);

    struct Api;

    #[OpenApi]
    impl Api {
        #[oai(path = "/", method = "get")]
        async fn test(&self) -> Ok {
            Ok(Binary(b"{ \"value\": 100 }".to_vec()))
        }
    }

    let meta: MetaApi = Api::meta().remove(0);
    assert_eq!(meta.paths[0].path, "/");

    let operator = &meta.paths[0].operations[0];
    let response = &operator.responses.responses[0];

    assert_eq!(response.status, Some(200));

    let media = &response.content[0];
    assert_eq!(media.content_type, "application/json; charset=utf-8");
    assert_eq!(media.schema, <Json<MyObj>>::schema_ref());

    let ep = OpenApiService::new(Api, "test", "1.0");
    let cli = TestClient::new(ep);
    let resp = cli.get("/").send().await;

    resp.assert_content_type("application/json; charset=utf-8");
    resp.assert_json(&serde_json::json!({ "value": 100 })).await;

    let mut registry = Registry::new();
    Api::register(&mut registry);
    let type_name: Vec<&String> = registry.schemas.keys().collect();
    assert_eq!(&type_name, &["MyObj"]);
}
