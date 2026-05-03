use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::util::ServiceExt;
use rustbackendstarter::create_app;

#[tokio::test]
async fn test_404_not_found() {
    let app = create_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/non-existent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
