use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{error::AppError, middleware::auth::Claims};

#[derive(Debug, Serialize, ToSchema)]
pub struct DashboardResponse {
    pub user_id: String,
    pub role: String,
    pub message: String,
}

/// Get user dashboard data (Requires JWT)
#[utoipa::path(
    get,
    path = "/dashboard",
    responses(
        (status = 200, description = "Dashboard data", body = DashboardResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn dashboard(claims: Claims) -> Result<Json<DashboardResponse>, AppError> {
    Ok(Json(DashboardResponse {
        user_id: claims.sub.clone(),
        role: claims.role.clone(),
        message: format!("Welcome, {}! You are logged in.", claims.role),
    }))
}
