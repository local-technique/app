use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::common::error::AppError;

pub async fn connect(database_url: &str) -> Result<PgPool, AppError> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(|e| AppError::internal(format!("failed to connect to database: {e}")))
}

pub async fn migrate(pool: &PgPool) -> Result<(), AppError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| AppError::internal(format!("failed to run migrations: {e}")))
}
