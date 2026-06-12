use sqlx::PgPool;
use uuid::Uuid;

use crate::api_tokens::model::ApiToken;

pub async fn insert_token(
    db: &PgPool,
    id: Uuid,
    user_id: Uuid,
    token_prefix: &str,
    token_hash: &str,
) -> Result<ApiToken, sqlx::Error> {
    sqlx::query_as::<_, ApiToken>(
        r#"
        INSERT INTO api_tokens (id, user_id, token_prefix, token_hash)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, token_prefix, token_hash, created_at, last_used_at
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(token_prefix)
    .bind(token_hash)
    .fetch_one(db)
    .await
}

pub async fn find_active_token(
    db: &PgPool,
    user_id: Uuid,
) -> Result<Option<ApiToken>, sqlx::Error> {
    sqlx::query_as::<_, ApiToken>(
        r#"
        SELECT id, user_id, token_prefix, token_hash, created_at, last_used_at
        FROM api_tokens
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
}

pub async fn find_token_by_hash(
    db: &PgPool,
    token_hash: &str,
) -> Result<Option<ApiToken>, sqlx::Error> {
    sqlx::query_as::<_, ApiToken>(
        r#"
        SELECT id, user_id, token_prefix, token_hash, created_at, last_used_at
        FROM api_tokens
        WHERE token_hash = $1
        "#,
    )
    .bind(token_hash)
    .fetch_optional(db)
    .await
}

pub async fn delete_token(db: &PgPool, user_id: Uuid) -> Result<u64, sqlx::Error> {
    sqlx::query("DELETE FROM api_tokens WHERE user_id = $1")
        .bind(user_id)
        .execute(db)
        .await
        .map(|r| r.rows_affected())
}

pub async fn update_last_used(db: &PgPool, token_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE api_tokens SET last_used_at = now() WHERE id = $1")
        .bind(token_id)
        .execute(db)
        .await?;
    Ok(())
}
