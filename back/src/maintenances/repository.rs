use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::maintenances::model::{
    MaintenanceDetail, MaintenanceListItem, MaintenanceTranslationMatrixRow, MaintenanceTranslationValue,
    MaintenanceUpsertRequest,
};

pub async fn list(
    db: &sqlx::PgPool,
    locale_chain: &[String],
    query: Option<&str>,
) -> Result<Vec<MaintenanceListItem>, AppError> {
    let rows = sqlx::query(
        r#"
SELECT
  m.id,
  m.code,
  m.category_code,
  m.start_utc,
  m.end_utc,
  m.notified_at_utc,
  coalesce((
    SELECT mi.field_value
    FROM maintenance_i18n mi
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = mi.locale
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'title'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS title,
  coalesce((
    SELECT mi.field_value
    FROM maintenance_i18n mi
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = mi.locale
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'warning'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS warning,
  coalesce((
    SELECT mi.field_value
    FROM maintenance_i18n mi
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = mi.locale
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'short_description'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS short_description,
  coalesce((
    SELECT mi.field_value
    FROM maintenance_i18n mi
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = mi.locale
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'location'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS location
FROM maintenances m
WHERE ($2::TEXT IS NULL OR $2 = '') OR (
  coalesce((
    SELECT mi.field_value
    FROM maintenance_i18n mi
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = mi.locale
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'title'
    ORDER BY lp.ord
    LIMIT 1
  ), '') ILIKE ('%' || $2 || '%')
  OR coalesce((
    SELECT mi.field_value
    FROM maintenance_i18n mi
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = mi.locale
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'short_description'
    ORDER BY lp.ord
    LIMIT 1
  ), '') ILIKE ('%' || $2 || '%')
  OR m.category_code ILIKE ('%' || $2 || '%')
)
ORDER BY m.start_utc ASC
"#,
    )
    .bind(locale_chain)
    .bind(query)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(to_list_item).collect()
}

fn to_list_item(row: sqlx::postgres::PgRow) -> Result<MaintenanceListItem, AppError> {
    let id: String = row.try_get("code")?;
    let start_utc: DateTime<Utc> = row.try_get("start_utc")?;
    let end_utc: Option<DateTime<Utc>> = row.try_get("end_utc")?;
    let notified_at_utc: Option<DateTime<Utc>> = row.try_get("notified_at_utc")?;
    Ok(MaintenanceListItem {
        id,
        category_code: row.try_get("category_code")?,
        title: row.try_get("title")?,
        warning: row.try_get("warning")?,
        short_description: row.try_get("short_description")?,
        location: row.try_get("location")?,
        start_utc: start_utc.to_rfc3339(),
        end_utc: end_utc.map(|value| value.to_rfc3339()),
        notified_at_utc: notified_at_utc.map(|value| value.to_rfc3339()),
    })
}

pub async fn by_id(
    db: &sqlx::PgPool,
    maintenance_code: &str,
    locale_chain: &[String],
) -> Result<Option<MaintenanceDetail>, AppError> {
    let row = sqlx::query(
        r#"
SELECT
  m.id,
  m.code,
  m.category_code,
  m.start_utc,
  m.end_utc,
  m.notified_at_utc,
  coalesce((
    SELECT mi.field_value
    FROM maintenance_i18n mi
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = mi.locale
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'title'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS title,
  coalesce((
    SELECT mi.field_value
    FROM maintenance_i18n mi
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = mi.locale
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'warning'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS warning,
  coalesce((
    SELECT mi.field_value
    FROM maintenance_i18n mi
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = mi.locale
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'short_description'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS short_description,
  coalesce((
    SELECT mi.field_value
    FROM maintenance_i18n mi
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = mi.locale
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'long_description'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS long_description,
  coalesce((
    SELECT mi.field_value
    FROM maintenance_i18n mi
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = mi.locale
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'location'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS location
FROM maintenances m
WHERE m.code = $1
"#,
    )
    .bind(maintenance_code)
    .bind(locale_chain)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let code: String = row.try_get("code")?;
    let start_utc: DateTime<Utc> = row.try_get("start_utc")?;
    let end_utc: Option<DateTime<Utc>> = row.try_get("end_utc")?;
    let notified_at_utc: Option<DateTime<Utc>> = row.try_get("notified_at_utc")?;
    Ok(Some(MaintenanceDetail {
        id: code,
        category_code: row.try_get("category_code")?,
        title: row.try_get("title")?,
        warning: row.try_get("warning")?,
        short_description: row.try_get("short_description")?,
        long_description: row.try_get("long_description")?,
        location: row.try_get("location")?,
        start_utc: start_utc.to_rfc3339(),
        end_utc: end_utc.map(|value| value.to_rfc3339()),
        notified_at_utc: notified_at_utc.map(|value| value.to_rfc3339()),
    }))
}

pub async fn list_translations(
    db: &sqlx::PgPool,
    maintenance_code: &str,
) -> Result<Vec<MaintenanceTranslationMatrixRow>, AppError> {
    let maintenance_exists: Option<bool> = sqlx::query_scalar("SELECT TRUE FROM maintenances WHERE code = $1")
        .bind(maintenance_code)
        .fetch_optional(db)
        .await?;
    if maintenance_exists.is_none() {
        return Err(AppError::not_found("maintenance not found"));
    }

    let rows = sqlx::query(
        r#"
WITH target AS (
  SELECT id FROM maintenances WHERE code = $1
)
SELECT
  k.field_key,
  l.code AS locale,
  mi.field_value
FROM locales l
CROSS JOIN (
  SELECT DISTINCT field_key
  FROM maintenance_i18n
  WHERE maintenance_id = (SELECT id FROM target)
) k
LEFT JOIN maintenance_i18n mi
  ON mi.maintenance_id = (SELECT id FROM target)
 AND mi.locale = l.code
 AND mi.field_key = k.field_key
ORDER BY k.field_key, l.code
"#,
    )
    .bind(maintenance_code)
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(MaintenanceTranslationMatrixRow {
                field_key: row.try_get("field_key")?,
                locale: row.try_get("locale")?,
                field_value: row.try_get("field_value")?,
            })
        })
        .collect()
}

pub async fn replace_translations(
    db: &sqlx::PgPool,
    maintenance_code: &str,
    values: &[MaintenanceTranslationValue],
) -> Result<(), AppError> {
    let maintenance_id: Uuid = sqlx::query_scalar("SELECT id FROM maintenances WHERE code = $1")
        .bind(maintenance_code)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::not_found("maintenance not found"))?;

    let mut tx = db.begin().await?;

    sqlx::query("DELETE FROM maintenance_i18n WHERE maintenance_id = $1")
        .bind(maintenance_id)
        .execute(&mut *tx)
        .await?;

    for value in values {
        sqlx::query(
            "INSERT INTO maintenance_i18n (maintenance_id, locale, field_key, field_value) VALUES ($1, $2, $3, $4)",
        )
        .bind(maintenance_id)
        .bind(value.locale.trim().to_lowercase())
        .bind(value.field_key.trim().to_lowercase())
        .bind(value.field_value.trim())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn upsert(db: &sqlx::PgPool, payload: &MaintenanceUpsertRequest) -> Result<(), AppError> {
    let start_utc = DateTime::parse_from_rfc3339(&payload.start_utc)
        .map_err(|_| AppError::bad_request("invalid maintenance start_utc"))?
        .with_timezone(&Utc);
    let end_utc = payload
        .end_utc
        .as_deref()
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|_| AppError::bad_request("invalid maintenance end_utc"))?
        .map(|value| value.with_timezone(&Utc));
    let notified_at_utc = payload
        .notified_at_utc
        .as_deref()
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|_| AppError::bad_request("invalid maintenance notified_at_utc"))?
        .map(|value| value.with_timezone(&Utc));

    let mut tx = db.begin().await?;

    let maintenance_id: Uuid = sqlx::query_scalar::<_, Uuid>(
        r#"
INSERT INTO maintenances (id, code, category_code, start_utc, end_utc, notified_at_utc, updated_at)
VALUES ($1, $2, $3, $4, $5, $6, now())
ON CONFLICT (code) DO UPDATE
SET category_code = EXCLUDED.category_code,
    start_utc = EXCLUDED.start_utc,
    end_utc = EXCLUDED.end_utc,
    notified_at_utc = EXCLUDED.notified_at_utc,
    updated_at = now()
RETURNING id
"#,
    )
    .bind(Uuid::new_v4())
    .bind(payload.id.as_str())
    .bind(payload.category_code.as_str())
    .bind(start_utc)
    .bind(end_utc)
    .bind(notified_at_utc)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM maintenance_i18n WHERE maintenance_id = $1")
        .bind(maintenance_id)
        .execute(&mut *tx)
        .await?;

    for value in &payload.translations {
        sqlx::query(
            "INSERT INTO maintenance_i18n (maintenance_id, locale, field_key, field_value) VALUES ($1, $2, $3, $4)",
        )
        .bind(maintenance_id)
        .bind(value.locale.trim().to_lowercase())
        .bind(value.field_key.trim().to_lowercase())
        .bind(value.field_value.trim())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn delete_by_code(db: &sqlx::PgPool, maintenance_code: &str) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM maintenances WHERE code = $1")
        .bind(maintenance_code)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
}
