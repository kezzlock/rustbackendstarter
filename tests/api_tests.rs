use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use rustbackendstarter::create_app;
use rustbackendstarter::routes::auth::{
    LoginRequest, RegisterRequest, RegisterResponse, TokenPairResponse,
};
use serde_json::{json, Value};
use tower::util::ServiceExt;

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

#[tokio::test]
async fn test_full_auth_flow() {
    let app = create_app().await;

    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());
    let password = "securepassword123";

    // 1. Register
    let register_payload = RegisterRequest {
        email: email.clone(),
        password: password.to_string(),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&register_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let reg_res: RegisterResponse = serde_json::from_slice(&body).unwrap();
    assert!(!reg_res.user_id.is_nil());

    // 2. Login
    let login_payload = LoginRequest {
        email: email.clone(),
        password: password.to_string(),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let tokens: TokenPairResponse = serde_json::from_slice(&body).unwrap();
    assert!(!tokens.access_token.is_empty());

    // 3. Access Protected Dashboard
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/dashboard")
                .header("Authorization", format!("Bearer {}", tokens.access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let dash_res: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(dash_res["role"], "user");
    
    // Wait a second to ensure the refreshed token has a different timestamp
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // 4. Refresh Token
    let refresh_payload = json!({
        "refresh_token": tokens.refresh_token
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/refresh")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&refresh_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let new_tokens: TokenPairResponse = serde_json::from_slice(&body).unwrap();
    assert_ne!(tokens.access_token, new_tokens.access_token);
}

#[tokio::test]
async fn test_login_failure() {
    let app = create_app().await;

    let login_payload = json!({
        "email": "nonexistent@example.com",
        "password": "wrongpassword"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
