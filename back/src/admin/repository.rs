use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::common::role::Role;

#[derive(Debug, Clone)]
pub struct AdminUserRow {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub roles: Vec<String>,
}

pub struct ListUsersParams<'a> {
    pub offset: i64,
    pub limit: i64,
    pub search_email: Option<&'a str>,
    pub role: Option<&'a str>,
    pub only_without_roles: bool,
    pub sort: &'a str,
    pub direction: &'a str,
}

pub async fn list_users(
    db: &sqlx::PgPool,
    params: ListUsersParams<'_>,
) -> Result<(Vec<AdminUserRow>, i64), AppError> {
    let order_column = match params.sort {
        "email" => "email",
        "created_at" => "created_at",
        "last_login_at" => "last_login_at",
        _ => "id",
    };
    let order_direction = if params.direction == "desc" { "DESC" } else { "ASC" };
    let items_sql = format!(
        r#"
SELECT id, email::TEXT AS email, first_name, last_name, created_at, last_login_at, roles
FROM users
WHERE ($1::TEXT IS NULL OR email::TEXT ILIKE '%' || $1 || '%')
  AND (($3::BOOLEAN = TRUE AND cardinality(roles) = 0) OR ($3::BOOLEAN = FALSE AND ($2::TEXT IS NULL OR $2 = ANY(roles))))
ORDER BY {order_column} {order_direction} NULLS LAST, id ASC
OFFSET $4
LIMIT $5
"#
    );

    let total: i64 = sqlx::query_scalar(
        r#"
SELECT COUNT(*)
FROM users
WHERE ($1::TEXT IS NULL OR email::TEXT ILIKE '%' || $1 || '%')
  AND (($3::BOOLEAN = TRUE AND cardinality(roles) = 0) OR ($3::BOOLEAN = FALSE AND ($2::TEXT IS NULL OR $2 = ANY(roles))))
"#,
    )
    .bind(params.search_email)
    .bind(params.role)
    .bind(params.only_without_roles)
    .fetch_one(db)
    .await?;

    let rows = sqlx::query(&items_sql)
        .bind(params.search_email)
        .bind(params.role)
        .bind(params.only_without_roles)
        .bind(params.offset)
        .bind(params.limit)
        .fetch_all(db)
        .await?;

    let users = rows
        .into_iter()
        .map(|row| {
            Ok(AdminUserRow {
                id: row.try_get("id")?,
                email: row.try_get("email")?,
                first_name: row.try_get("first_name")?,
                last_name: row.try_get("last_name")?,
                created_at: row.try_get("created_at")?,
                last_login_at: row.try_get("last_login_at")?,
                roles: row.try_get("roles")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok((users, total))
}

pub async fn replace_non_admin_roles(
    db: &sqlx::PgPool,
    user_id: Uuid,
    non_admin_roles: &[String],
) -> Result<Option<Vec<String>>, AppError> {
    let row = sqlx::query(
        r#"
UPDATE users
SET roles = CASE
    WHEN $3 = ANY(roles) THEN array_prepend($3, $2::TEXT[])
    ELSE $2::TEXT[]
  END,
  updated_at = now()
WHERE id = $1
RETURNING roles
"#,
    )
    .bind(user_id)
    .bind(non_admin_roles)
    .bind(Role::Admin.code())
    .fetch_optional(db)
    .await?;

    row.map(|row| row.try_get("roles")).transpose().map_err(Into::into)
}

pub async fn update_user_names(
    db: &sqlx::PgPool,
    user_id: Uuid,
    first_name: Option<String>,
    last_name: Option<String>,
) -> Result<Option<crate::admin::model::UpdateUserNamesResponse>, AppError> {
    let row = sqlx::query(
        r#"
UPDATE users
SET first_name = $2, last_name = $3, updated_at = now()
WHERE id = $1
RETURNING id, first_name, last_name
"#,
    )
    .bind(user_id)
    .bind(&first_name)
    .bind(&last_name)
    .fetch_optional(db)
    .await?;

    row.map(|row| -> Result<crate::admin::model::UpdateUserNamesResponse, sqlx::Error> {
        Ok(crate::admin::model::UpdateUserNamesResponse {
            id: row.try_get::<Uuid, _>("id")?.to_string(),
            first_name: row.try_get("first_name")?,
            last_name: row.try_get("last_name")?,
        })
    })
    .transpose()
    .map_err(Into::into)
}
