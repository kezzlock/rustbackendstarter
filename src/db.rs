use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::{config::AppConfig, error::AppError};

pub async fn create_pool(config: &AppConfig) -> Result<PgPool, AppError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to connect to database: {e}")))?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), AppError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Migration failed: {e}")))?;
    Ok(())
}

/// Ensure an admin account exists. Creates one from env config if missing.
pub async fn seed_admin(pool: &PgPool, config: &AppConfig) -> Result<(), AppError> {
    use argon2::{
        password_hash::{PasswordHasher, SaltString},
        Argon2,
    };

    let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE role = 'admin'")
        .fetch_one(pool)
        .await?;

    if exists.0 > 0 {
        return Ok(());
    }

    tracing::info!("No admin found — seeding default admin account");

    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(config.admin_password.as_bytes(), &salt)
        .map_err(|e| AppError::Argon2(e.to_string()))?
        .to_string();

    let id = uuid::Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email, password_hash, role) VALUES ($1, $2, $3, 'admin')")
        .bind(id)
        .bind(&config.admin_email)
        .bind(password_hash)
        .execute(pool)
        .await?;

    tracing::info!(email = %config.admin_email, "Admin account created");
    Ok(())
}
