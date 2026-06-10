use std::collections::HashMap;

use axum::http::StatusCode;
use sqlx::Row;

use crate::categories::model::CategoryItem;
use crate::common::error::AppError;

const CATEGORY_REFERENCE_COUNT_QUERY: &str = r#"
SELECT
  (SELECT count(*) FROM incidents WHERE category_id = $1::uuid) +
  (SELECT count(*) FROM maintenances WHERE category_id = $1::uuid) +
  (SELECT count(*) FROM projects WHERE category_id = $1::uuid)
"#;

pub async fn list(db: &sqlx::PgPool, locale_chain: &[String]) -> Result<Vec<CategoryItem>, AppError> {
    let rows = sqlx::query(
        r#"
SELECT
  c.id::TEXT AS id,
  c.key,
  c.icon,
  c.color,
  coalesce((
    SELECT ci.label
    FROM event_category_i18n ci
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ci.locale
    WHERE ci.category_id = c.id
    ORDER BY lp.ord
    LIMIT 1
  ), c.key) AS label
FROM event_categories c
ORDER BY c.key ASC
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
            key: row.try_get("key")?,
            icon: row.try_get("icon")?,
            color: row.try_get("color")?,
            label: row.try_get("label")?,
        });
    }
    Ok(items)
}

pub async fn create(
    db: &sqlx::PgPool,
    key: &str,
    icon: &str,
    color: &str,
    labels: &HashMap<String, String>,
    locale_chain: &[String],
) -> Result<CategoryItem, AppError> {
    let mut tx = db.begin().await?;
    let row = sqlx::query(
        "INSERT INTO event_categories (id, key, icon, color) VALUES (gen_random_uuid(), $1, $2, $3) RETURNING id::TEXT",
    )
    .bind(key)
    .bind(icon)
    .bind(color)
    .fetch_one(&mut *tx)
    .await?;
    let id: String = row.try_get("id")?;
    replace_labels(&mut tx, &id, labels).await?;
    tx.commit().await?;
    find_by_id(db, &id, locale_chain).await
}

pub async fn update(
    db: &sqlx::PgPool,
    id: &str,
    key: &str,
    icon: &str,
    color: &str,
    labels: &HashMap<String, String>,
    locale_chain: &[String],
) -> Result<CategoryItem, AppError> {
    let mut tx = db.begin().await?;
    let result = sqlx::query("UPDATE event_categories SET key = $2, icon = $3, color = $4, updated_at = now() WHERE id = $1::uuid")
        .bind(id)
        .bind(key)
        .bind(icon)
        .bind(color)
        .execute(&mut *tx)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("category not found"));
    }
    replace_labels(&mut tx, id, labels).await?;
    tx.commit().await?;
    find_by_id(db, id, locale_chain).await
}

pub async fn delete(db: &sqlx::PgPool, id: &str) -> Result<(), AppError> {
    let references: i64 = sqlx::query_scalar(CATEGORY_REFERENCE_COUNT_QUERY)
    .bind(id)
    .fetch_one(db)
    .await?;
    if references > 0 {
        return Err(AppError {
            status: StatusCode::CONFLICT,
            message: "category is referenced by events".to_string(),
        });
    }

    let result = sqlx::query("DELETE FROM event_categories WHERE id = $1::uuid")
        .bind(id)
        .execute(db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("category not found"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn delete_reference_count_includes_projects() {
        assert!(super::CATEGORY_REFERENCE_COUNT_QUERY.contains("FROM projects WHERE category_id = $1::uuid"));
    }
}

async fn find_by_id(db: &sqlx::PgPool, id: &str, locale_chain: &[String]) -> Result<CategoryItem, AppError> {
    let row = sqlx::query(
        r#"
SELECT
  c.id::TEXT AS id,
  c.key,
  c.icon,
  c.color,
  coalesce((
    SELECT ci.label
    FROM event_category_i18n ci
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ci.locale
    WHERE ci.category_id = c.id
    ORDER BY lp.ord
    LIMIT 1
  ), c.key) AS label
FROM event_categories c
WHERE c.id = $1::uuid
"#,
    )
    .bind(id)
    .bind(locale_chain)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::not_found("category not found"))?;

    Ok(CategoryItem {
        labels: labels_for(db, id).await?,
        id: row.try_get("id")?,
        key: row.try_get("key")?,
        icon: row.try_get("icon")?,
        color: row.try_get("color")?,
        label: row.try_get("label")?,
    })
}

async fn labels_for(db: &sqlx::PgPool, id: &str) -> Result<HashMap<String, String>, AppError> {
    let rows = sqlx::query("SELECT locale, label FROM event_category_i18n WHERE category_id = $1::uuid")
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
    sqlx::query("DELETE FROM event_category_i18n WHERE category_id = $1::uuid")
        .bind(id)
        .execute(&mut **tx)
        .await?;
    for (locale, label) in labels {
        sqlx::query("INSERT INTO event_category_i18n (category_id, locale, label) VALUES ($1::uuid, $2, $3)")
            .bind(id)
            .bind(locale)
            .bind(label)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}
