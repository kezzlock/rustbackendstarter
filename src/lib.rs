pub mod config;
pub mod db;
pub mod error;
pub mod middleware;
pub mod models;
pub mod routes;

use axum::{
    http::{HeaderValue, Method},
    routing::{get, post},
    Router,
    Extension,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use config::AppConfig;
use routes::{
    admin::{AdminState, list_users},
    auth::{AuthState, login, refresh, register},
    dashboard::dashboard,
    health::{health_check, root, HealthResponse},
};

// ─── OpenAPI Documentation ───────────────────────────────────────────────────

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health::root,
        routes::health::health_check,
        routes::auth::register,
        routes::auth::login,
        routes::auth::refresh,
        routes::dashboard::dashboard,
        routes::admin::list_users,
    ),
    components(
        schemas(
            routes::health::HealthResponse,
            routes::auth::RegisterRequest,
            routes::auth::RegisterResponse,
            routes::auth::LoginRequest,
            routes::auth::TokenPairResponse,
            routes::auth::RefreshRequest,
            routes::dashboard::DashboardResponse,
            routes::admin::UsersResponse,
            routes::admin::UserListItem,
        )
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

// ─── App Creation ─────────────────────────────────────────────────────────────

/// Creates the Axum router with all routes and middleware.
pub async fn create_app() -> Router {
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env().expect("Failed to load configuration");

    let pool = db::create_pool(&config)
        .await
        .expect("Failed to create database pool");

    db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    db::seed_admin(&pool, &config)
        .await
        .expect("Failed to seed admin account");

    let cors = {
        let origins: Vec<HeaderValue> = config
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse::<HeaderValue>().ok())
            .collect();

        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(tower_http::cors::Any)
    };

    let auth_state = AuthState {
        pool: pool.clone(),
        jwt_secret: config.jwt_secret.clone(),
    };
    let admin_state = AdminState {
        pool: pool.clone(),
    };

    let jwt_extension = Extension(config.jwt_secret.clone());

    let auth_router = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .with_state(auth_state);

    let admin_router = Router::new()
        .route("/users", get(list_users))
        .layer(jwt_extension.clone())
        .with_state(admin_state);

    let dashboard_router = Router::new()
        .route("/dashboard", get(dashboard))
        .layer(jwt_extension);

    Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(root))
        .route("/health", get(health_check).with_state(pool))
        .nest("/auth", auth_router)
        .nest("/admin", admin_router)
        .merge(dashboard_router)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
