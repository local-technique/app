use std::collections::HashMap;

use axum::http::StatusCode;
use sqlx::Row;

use crate::categories::model::CategoryItem;
use crate::common::error::AppError;

pub async fn list(db: &sqlx::PgPool, locale_chain: &[String]) -> Result<Vec<CategoryItem>, AppError> {
    let rows = sqlx::query(
        r#"
SELECT
  c.id,
  c.code,
  c.icon,
  coalesce((
    SELECT ci.label
    FROM event_category_i18n ci
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ci.locale
    WHERE ci.category_id = c.id
    ORDER BY lp.ord
    LIMIT 1
  ), c.code) AS label
FROM event_categories c
ORDER BY c.code ASC
"#,
    )
    .bind(locale_chain)
    .fetch_all(db)
    .await?;

    let mut items = Vec::new();
    for row in rows {
        let id: String = row.try_get("id")?;
        items.push(CategoryItem {
            labels: labels_for(db, &id).await?,
            id,
            code: row.try_get("code")?,
            icon: row.try_get("icon")?,
            label: row.try_get("label")?,
        });
    }
    Ok(items)
}

pub async fn create(
    db: &sqlx::PgPool,
    id: &str,
    code: &str,
    icon: &str,
    labels: &HashMap<String, String>,
) -> Result<(), AppError> {
    let mut tx = db.begin().await?;
    sqlx::query("INSERT INTO event_categories (id, code, icon) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(code)
        .bind(icon)
        .execute(&mut *tx)
        .await?;
    replace_labels(&mut tx, id, labels).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn update(
    db: &sqlx::PgPool,
    id: &str,
    code: &str,
    icon: &str,
    labels: &HashMap<String, String>,
) -> Result<(), AppError> {
    let mut tx = db.begin().await?;
    let result = sqlx::query("UPDATE event_categories SET code = $2, icon = $3, updated_at = now() WHERE id = $1")
        .bind(id)
        .bind(code)
        .bind(icon)
        .execute(&mut *tx)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("category not found"));
    }
    replace_labels(&mut tx, id, labels).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn delete(db: &sqlx::PgPool, id: &str) -> Result<(), AppError> {
    let references: i64 = sqlx::query_scalar(
        r#"
SELECT
  (SELECT count(*) FROM incidents WHERE category_code = $1) +
  (SELECT count(*) FROM maintenances WHERE category_code = $1)
"#,
    )
    .bind(id)
    .fetch_one(db)
    .await?;
    if references > 0 {
        return Err(AppError {
            status: StatusCode::CONFLICT,
            message: "category is referenced by events".to_string(),
        });
    }

    let result = sqlx::query("DELETE FROM event_categories WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("category not found"));
    }
    Ok(())
}

async fn labels_for(db: &sqlx::PgPool, id: &str) -> Result<HashMap<String, String>, AppError> {
    let rows = sqlx::query("SELECT locale, label FROM event_category_i18n WHERE category_id = $1")
        .bind(id)
        .fetch_all(db)
        .await?;
    rows.into_iter()
        .map(|row| Ok((row.try_get("locale")?, row.try_get("label")?)))
        .collect()
}

async fn replace_labels(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id: &str,
    labels: &HashMap<String, String>,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM event_category_i18n WHERE category_id = $1")
        .bind(id)
        .execute(&mut **tx)
        .await?;
    for (locale, label) in labels {
        sqlx::query("INSERT INTO event_category_i18n (category_id, locale, label) VALUES ($1, $2, $3)")
            .bind(id)
            .bind(locale)
            .bind(label)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}
