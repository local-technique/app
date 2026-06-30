use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::auth::model::Provider;
use crate::common::error::AppError;
use crate::common::role::Role;

#[derive(Debug, Clone)]
pub struct DbSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub compromised_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct RotatedSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DbUser {
    pub id: Uuid,
    pub provider: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub roles: Vec<String>,
}

pub async fn mark_user_login(db: &sqlx::PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET last_login_at = now(), updated_at = now() WHERE id = $1")
        .bind(user_id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn find_or_create_user(
    db: &sqlx::PgPool,
    provider: Provider,
    email: &str,
    first_name: Option<&str>,
    last_name: Option<&str>,
) -> Result<DbUser, AppError> {
    for _ in 0..3 {
        if let Some(user) = find_user_by_email(db, email).await? {
            return Ok(user);
        }

        let result = sqlx::query(
            r#"
INSERT INTO users (id, email, provider, first_name, last_name)
VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (email) DO NOTHING
RETURNING id, provider, email, first_name, last_name, roles
"#,
        )
        .bind(Uuid::new_v4())
        .bind(email)
        .bind(provider.as_str())
        .bind(first_name)
        .bind(last_name)
        .fetch_optional(db)
        .await?;

        if let Some(row) = result {
            return Ok(DbUser {
                id: row.try_get("id")?,
                provider: row.try_get("provider")?,
                email: row.try_get("email")?,
                first_name: row.try_get("first_name")?,
                last_name: row.try_get("last_name")?,
                roles: row.try_get("roles")?,
            });
        }
    }

    Err(AppError::internal("failed to create user after retries"))
}

async fn find_user_by_email(db: &sqlx::PgPool, email: &str) -> Result<Option<DbUser>, AppError> {
    let row = sqlx::query(
        r#"
SELECT id, provider, email, first_name, last_name, roles
FROM users
WHERE email = $1
"#,
    )
    .bind(email)
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => Ok(Some(DbUser {
            id: row.try_get("id")?,
            provider: row.try_get("provider")?,
            email: row.try_get("email")?,
            first_name: row.try_get("first_name")?,
            last_name: row.try_get("last_name")?,
            roles: row.try_get("roles")?,
        })),
        None => Ok(None),
    }
}

pub async fn ensure_admin_role(db: &sqlx::PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        r#"
UPDATE users
SET roles = CASE
  WHEN NOT ($2 = ANY(roles)) THEN array_append(roles, $2)
  ELSE roles
END,
updated_at = now()
WHERE id = $1
"#,
    )
    .bind(user_id)
    .bind(Role::Admin.code())
    .execute(db)
    .await?;

    Ok(())
}

pub async fn get_user_by_id(db: &sqlx::PgPool, user_id: Uuid) -> Result<Option<DbUser>, AppError> {
    let row = sqlx::query("SELECT id, provider, email, first_name, last_name, roles FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(db)
        .await?;

    row.map(|row| {
        Ok(DbUser {
            id: row.try_get("id")?,
            provider: row.try_get("provider")?,
            email: row.try_get("email")?,
            first_name: row.try_get("first_name")?,
            last_name: row.try_get("last_name")?,
            roles: row.try_get("roles")?,
        })
    })
    .transpose()
}

pub async fn insert_session(
    db: &sqlx::PgPool,
    user_id: Uuid,
    refresh_token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<Uuid, AppError> {
    let session_id = Uuid::new_v4();
    sqlx::query(
        r#"
INSERT INTO auth_sessions (id, user_id, refresh_token_hash, expires_at)
VALUES ($1, $2, $3, $4)
"#,
    )
    .bind(session_id)
    .bind(user_id)
    .bind(refresh_token_hash)
    .bind(expires_at)
    .execute(db)
    .await?;
    Ok(session_id)
}

pub async fn find_session_by_refresh_hash(
    db: &sqlx::PgPool,
    refresh_hash: &str,
) -> Result<Option<DbSession>, AppError> {
    let row = sqlx::query(
        r#"
SELECT id, user_id, refresh_token_hash, previous_refresh_token_hashes, expires_at, revoked_at, compromised_at
FROM auth_sessions
WHERE refresh_token_hash = $1
"#,
    )
    .bind(refresh_hash)
    .fetch_optional(db)
    .await?;

    row.map(|row| {
        Ok(DbSession {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            expires_at: row.try_get("expires_at")?,
            revoked_at: row.try_get("revoked_at")?,
            compromised_at: row.try_get("compromised_at")?,
        })
    })
    .transpose()
}

pub async fn find_session_by_previous_refresh_hash(
    db: &sqlx::PgPool,
    refresh_hash: &str,
) -> Result<Option<DbSession>, AppError> {
    let row = sqlx::query(
        r#"
SELECT id, user_id, refresh_token_hash, previous_refresh_token_hashes, expires_at, revoked_at, compromised_at
FROM auth_sessions
WHERE $1 = ANY(previous_refresh_token_hashes)
"#,
    )
    .bind(refresh_hash)
    .fetch_optional(db)
    .await?;

    row.map(|row| {
        Ok(DbSession {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            expires_at: row.try_get("expires_at")?,
            revoked_at: row.try_get("revoked_at")?,
            compromised_at: row.try_get("compromised_at")?,
        })
    })
    .transpose()
}

pub async fn find_session_by_id(db: &sqlx::PgPool, session_id: Uuid) -> Result<Option<DbSession>, AppError> {
    let row = sqlx::query(
        r#"
SELECT id, user_id, refresh_token_hash, previous_refresh_token_hashes, expires_at, revoked_at, compromised_at
FROM auth_sessions
WHERE id = $1
"#,
    )
    .bind(session_id)
    .fetch_optional(db)
    .await?;

    row.map(|row| {
        Ok(DbSession {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            expires_at: row.try_get("expires_at")?,
            revoked_at: row.try_get("revoked_at")?,
            compromised_at: row.try_get("compromised_at")?,
        })
    })
    .transpose()
}

pub async fn rotate_session_refresh_token(
    db: &sqlx::PgPool,
    incoming_refresh_hash: &str,
    next_refresh_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<Option<RotatedSession>, AppError> {
    let row = sqlx::query(
        r#"
WITH rotated AS (
  UPDATE auth_sessions s
  SET previous_refresh_token_hashes = (
        CASE
          WHEN array_length(previous_refresh_token_hashes, 1) IS NULL
            THEN ARRAY[refresh_token_hash]
          WHEN array_length(previous_refresh_token_hashes, 1) >= 8
            THEN previous_refresh_token_hashes[2:8] || refresh_token_hash
          ELSE previous_refresh_token_hashes || refresh_token_hash
        END
      ),
      refresh_token_hash = $2,
      expires_at = $3,
      updated_at = now()
  FROM users u
  WHERE s.refresh_token_hash = $1
    AND s.user_id = u.id
    AND s.revoked_at IS NULL
    AND s.compromised_at IS NULL
    AND s.expires_at > now()
  RETURNING s.id, s.user_id, u.provider, u.email, u.first_name, u.last_name, u.roles
),
_logged AS (
  UPDATE users SET last_login_at = now(), updated_at = now()
  WHERE id = (SELECT user_id FROM rotated)
)
SELECT id, user_id, provider, email, first_name, last_name, roles FROM rotated
"#,
    )
    .bind(incoming_refresh_hash)
    .bind(next_refresh_hash)
    .bind(expires_at)
    .fetch_optional(db)
    .await?;

    row.map(|row| {
        Ok(RotatedSession {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            provider: row.try_get("provider")?,
            email: row.try_get("email")?,
            first_name: row.try_get("first_name")?,
            last_name: row.try_get("last_name")?,
            roles: row.try_get("roles")?,
        })
    })
    .transpose()
}

pub async fn revoke_session(db: &sqlx::PgPool, session_id: Uuid) -> Result<(), AppError> {
    sqlx::query("UPDATE auth_sessions SET revoked_at = now(), updated_at = now() WHERE id = $1")
        .bind(session_id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn compromise_session(db: &sqlx::PgPool, session_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE auth_sessions SET compromised_at = now(), revoked_at = now(), updated_at = now() WHERE id = $1",
    )
    .bind(session_id)
    .execute(db)
    .await?;
    Ok(())
}
