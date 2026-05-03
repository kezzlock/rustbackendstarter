use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub admin_email: String,
    pub admin_password: String,
    pub allowed_origins: Vec<String>,
    pub port: u16,
}

impl AppConfig {
    /// Load config from environment variables. Call after dotenv().
    pub fn from_env() -> Result<Self, String> {
        dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set".to_string())?;

        let jwt_secret =
            env::var("JWT_SECRET").map_err(|_| "JWT_SECRET must be set".to_string())?;
        if jwt_secret.len() < 32 {
            return Err("JWT_SECRET must be at least 32 characters".to_string());
        }

        let admin_email =
            env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.com".to_string());

        let admin_password =
            env::var("ADMIN_PASSWORD").map_err(|_| "ADMIN_PASSWORD must be set".to_string())?;

        let allowed_origins_str =
            env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let allowed_origins = allowed_origins_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|_| "PORT must be a valid number".to_string())?;

        Ok(AppConfig {
            database_url,
            jwt_secret,
            admin_email,
            admin_password,
            allowed_origins,
            port,
        })
    }
}
