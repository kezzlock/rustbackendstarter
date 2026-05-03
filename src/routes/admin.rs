use axum::{
    extract::{Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{error::AppError, middleware::auth::AdminClaims};

#[derive(Clone)]
pub struct AdminState {
    pub pool: PgPool,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct PaginationParams {
    /// Page number (starting from 1)
    #[serde(default = "default_page")]
    pub page: i64,
    /// Items per page (max 100)
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}
fn default_per_page() -> i64 {
    20
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserListItem {
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UsersResponse {
    pub users: Vec<UserListItem>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}

#[derive(FromRow)]
struct UserDbRow {
    id: Uuid,
    email: String,
    role: String,
    created_at: DateTime<Utc>,
}

/// List all users (paginated, Admin only)
#[utoipa::path(
    get,
    path = "/admin/users",
    params(PaginationParams),
    responses(
        (status = 200, description = "List of users", body = UsersResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden (Admin only)")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_users(
    _claims: AdminClaims,
    State(state): State<AdminState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<UsersResponse>, AppError> {
    let per_page = params.per_page.clamp(1, 100);
    let page = params.page.max(1);
    let offset = (page - 1) * per_page;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await?;

    let rows: Vec<UserDbRow> = sqlx::query_as(
        "SELECT id, email, role, created_at FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    let users = rows
        .into_iter()
        .map(|r| UserListItem {
            id: r.id,
            email: r.email,
            role: r.role,
            created_at: r.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(UsersResponse {
        users,
        page,
        per_page,
        total: total.0,
    }))
}
