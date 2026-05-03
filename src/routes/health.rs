use crate::error::AppError;
use axum::{extract::State, Json};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
    pub version: String,
}

/// GET / — Basic info about the API
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Returns welcome message", body = String)
    )
)]
pub async fn root() -> &'static str {
    "Rust Backend Starter"
}

/// GET /health — System health information
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "System is healthy", body = HealthResponse),
        (status = 500, description = "System is unhealthy")
    )
)]
pub async fn health_check(State(pool): State<PgPool>) -> Result<Json<HealthResponse>, AppError> {
    // Check database connection
    let db_status = match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => "up".to_string(),
        Err(e) => format!("down: {}", e),
    };

    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        database: db_status,
        version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}
