use axum::{extract::State, http::StatusCode, Json};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::Rng;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow};
use validator::Validate;
use uuid::Uuid;
use utoipa::ToSchema;

use crate::{error::AppError, middleware::auth::Claims};

/// Shared app state for routes
#[derive(Clone)]
pub struct AuthState {
    pub pool: PgPool,
    pub jwt_secret: String,
}

// ─── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    /// User email address
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "user@example.com")]
    pub email: String,
    /// Password (min 8 characters)
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    #[schema(example = "securepassword123")]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "admin@example.com")]
    pub email: String,
    #[validate(length(min = 1, message = "Password is required"))]
    #[schema(example = "changeme123")]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RefreshRequest {
    #[validate(length(min = 1, message = "refresh_token is required"))]
    pub refresh_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TokenPairResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RegisterResponse {
    pub user_id: Uuid,
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

const ACCESS_TOKEN_SECS: i64 = 15 * 60;        // 15 minutes
const REFRESH_TOKEN_SECS: i64 = 7 * 24 * 3600; // 7 days

fn generate_access_token(user_id: &str, role: &str, jwt_secret: &str) -> Result<String, AppError> {
    let now = Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        iat: now,
        exp: now + ACCESS_TOKEN_SECS as usize,
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    ).map_err(AppError::Jwt)
}

fn generate_refresh_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

async fn store_refresh_token(
    pool: &PgPool,
    token: &str,
    user_id: Uuid,
) -> Result<(), AppError> {
    let expires_at = Utc::now() + chrono::Duration::seconds(REFRESH_TOKEN_SECS);
    sqlx::query(
        "INSERT INTO refresh_tokens (token, user_id, expires_at) VALUES ($1, $2, $3)"
    )
    .bind(token)
    .bind(user_id)
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(())
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// Register a new user
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = RegisterResponse),
        (status = 400, description = "Invalid input or email taken"),
        (status = 422, description = "Validation error")
    )
)]
pub async fn register(
    State(state): State<AuthState>,
    Json(body): Json<RegisterRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let existing: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE email = $1"
    )
    .bind(&body.email)
    .fetch_one(&state.pool)
    .await?;

    if existing.0 > 0 {
        return Err(AppError::BadRequest("Email already registered".to_string()));
    }

    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| AppError::Argon2(e.to_string()))?
        .to_string();

    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)"
    )
    .bind(id)
    .bind(&body.email)
    .bind(password_hash)
    .execute(&state.pool)
    .await?;

    tracing::info!(user_id = %id, "User registered");
    Ok((StatusCode::CREATED, Json(RegisterResponse { user_id: id })))
}

#[derive(FromRow)]
struct UserRow {
    id: Uuid,
    password_hash: String,
    role: String,
}

/// Login and receive access/refresh tokens
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = TokenPairResponse),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn login(
    State(state): State<AuthState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<TokenPairResponse>, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let row: UserRow = sqlx::query_as(
        "SELECT id, password_hash, role FROM users WHERE email = $1"
    )
    .bind(&body.email)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    let parsed_hash = PasswordHash::new(&row.password_hash)
        .map_err(|e| AppError::Argon2(e.to_string()))?;
    
    Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("Invalid credentials".to_string()))?;

    let user_id = row.id;
    let access_token = generate_access_token(&user_id.to_string(), &row.role, &state.jwt_secret)?;
    let refresh_token = generate_refresh_token();
    store_refresh_token(&state.pool, &refresh_token, user_id).await?;

    tracing::info!(user_id = %user_id, "User logged in");
    Ok(Json(TokenPairResponse {
        access_token,
        refresh_token,
        expires_in: ACCESS_TOKEN_SECS as u64,
    }))
}

#[derive(FromRow)]
struct RefreshRow {
    user_id: Uuid,
    role: String,
}

/// Refresh access token using a refresh token
#[utoipa::path(
    post,
    path = "/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed", body = TokenPairResponse),
        (status = 401, description = "Invalid or expired refresh token")
    )
)]
pub async fn refresh(
    State(state): State<AuthState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<TokenPairResponse>, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let now = Utc::now();

    let row: RefreshRow = sqlx::query_as(
        "SELECT rt.user_id, u.role FROM refresh_tokens rt \
         JOIN users u ON u.id = rt.user_id \
         WHERE rt.token = $1 AND rt.expires_at > $2"
    )
    .bind(&body.refresh_token)
    .bind(now)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid or expired refresh token".to_string()))?;

    sqlx::query(
        "DELETE FROM refresh_tokens WHERE token = $1"
    )
    .bind(&body.refresh_token)
    .execute(&state.pool)
    .await?;

    let user_id = row.user_id;
    let access_token = generate_access_token(&user_id.to_string(), &row.role, &state.jwt_secret)?;
    let new_refresh = generate_refresh_token();
    store_refresh_token(&state.pool, &new_refresh, user_id).await?;

    Ok(Json(TokenPairResponse {
        access_token,
        refresh_token: new_refresh,
        expires_in: ACCESS_TOKEN_SECS as u64,
    }))
}
