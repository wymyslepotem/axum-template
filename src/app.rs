use axum::{
    Router,
    http::header::{AUTHORIZATION, COOKIE, SET_COOKIE},
};
use tower::ServiceBuilder;
use tower_http::{
    LatencyUnit, ServiceBuilderExt,
    request_id::MakeRequestUuid,
    sensitive_headers::SetSensitiveHeadersLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};

use crate::state::AppState;

pub fn build_router() -> Router<AppState> {
    let routes = crate::routes::router();

    let middleware = ServiceBuilder::new()
        .layer(SetSensitiveHeadersLayer::new([
            AUTHORIZATION,
            COOKIE,
            SET_COOKIE,
        ]))
        .set_x_request_id(MakeRequestUuid)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(false))
                .on_response(DefaultOnResponse::new().latency_unit(LatencyUnit::Micros)),
        )
        .propagate_x_request_id();

    Router::<AppState>::new().merge(routes).layer(middleware)
}

#[cfg(test)]
mod tests {
    use super::build_router;

    use axum::{
        body::{Body, Bytes},
        http::{Request, StatusCode, header},
        response::{IntoResponse, Response},
    };
    use http_body_util::BodyExt;
    use serde_json::Value;
    use tower::util::ServiceExt;

    use crate::config::{AppEnv, LogFormat, Settings};
    use crate::state::AppState;

    fn test_settings() -> Settings {
        Settings {
            http_host: "127.0.0.1".parse().unwrap(),
            http_port: 0,
            app_env: AppEnv::Development,
            log_format: LogFormat::Pretty,
        }
    }

    fn test_app() -> axum::Router<()> {
        let state = AppState::new(test_settings());
        build_router().with_state(state)
    }

    async fn body_to_bytes(res: Response) -> Bytes {
        res.into_body()
            .collect()
            .await
            .expect("body collect must succeed")
            .to_bytes()
    }

    #[tokio::test]
    async fn health_returns_200_and_ok_true() {
        let app = test_app();

        let req = Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let bytes = body_to_bytes(res).await;
        let json: Value = serde_json::from_slice(bytes.as_ref()).unwrap();
        assert_eq!(json["ok"], true);
    }

    #[tokio::test]
    async fn request_id_is_generated_when_missing() {
        let app = test_app();

        let req = Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();

        let rid = res
            .headers()
            .get("x-request-id")
            .expect("x-request-id must be present")
            .to_str()
            .unwrap();

        assert!(!rid.is_empty(), "x-request-id must not be empty");
    }

    #[tokio::test]
    async fn request_id_is_propagated_when_provided() {
        let app = test_app();

        let req = Request::builder()
            .method("GET")
            .uri("/health")
            .header("x-request-id", "test-123")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();

        let rid = res
            .headers()
            .get("x-request-id")
            .expect("x-request-id must be present")
            .to_str()
            .unwrap();

        assert_eq!(rid, "test-123");
    }

    #[tokio::test]
    async fn openapi_contains_health_path() {
        let app = test_app();

        let req = Request::builder()
            .method("GET")
            .uri("/api-doc/openapi.json")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let bytes = body_to_bytes(res).await;
        let json: Value = serde_json::from_slice(bytes.as_ref()).unwrap();

        assert!(
            json["paths"]["/health"].is_object(),
            "OpenAPI must include /health"
        );
    }

    #[tokio::test]
    async fn swagger_ui_is_served() {
        let app = test_app();

        let req = Request::builder()
            .method("GET")
            .uri("/swagger-ui/")
            .header(header::ACCEPT, "text/html")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let bytes = body_to_bytes(res).await;
        let html = String::from_utf8_lossy(&bytes);

        assert!(
            html.to_lowercase().contains("swagger"),
            "Swagger UI HTML must contain 'swagger'"
        );
    }

    #[tokio::test]
    async fn app_error_bad_request_converts_to_json_response() {
        use axum::http::header::CONTENT_TYPE;

        let res = crate::error::AppError::bad_request("nope").into_response();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        let content_type = res
            .headers()
            .get(CONTENT_TYPE)
            .expect("content-type must be present")
            .to_str()
            .unwrap();

        assert!(
            content_type.starts_with("application/json"),
            "content-type must be application/json"
        );

        let bytes = body_to_bytes(res).await;
        let json: Value = serde_json::from_slice(bytes.as_ref()).unwrap();

        let body_str = json.to_string();
        assert!(body_str.contains("nope"), "error body must include message");
    }

    #[tokio::test]
    async fn app_error_internal_converts_to_500() {
        let res = crate::error::AppError::internal().into_response();
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
