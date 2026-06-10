use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::maintenances::model::{
    AuditUser, CategoryDisplay, EditFieldValue, MaintenanceDetail, MaintenanceEditData, MaintenanceListItem,
    MaintenanceSaveRequest, MaintenanceTranslationMatrixRow, MaintenanceTranslationValue,
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
  m.key,
  m.category_id::TEXT AS category_id,
  c.key AS category_display_key,
  c.icon AS category_icon,
  c.color AS category_color,
  coalesce((
    SELECT ci.label
    FROM event_category_i18n ci
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ci.locale
    WHERE ci.category_id = c.id
    ORDER BY lp.ord
    LIMIT 1
  ), c.key) AS category_label,
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
JOIN event_categories c ON c.id = m.category_id
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
  OR m.key ILIKE ('%' || $2 || '%')
  OR c.key ILIKE ('%' || $2 || '%')
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
    let key: String = row.try_get("key")?;
    let start_utc: DateTime<Utc> = row.try_get("start_utc")?;
    let end_utc: Option<DateTime<Utc>> = row.try_get("end_utc")?;
    let notified_at_utc: Option<DateTime<Utc>> = row.try_get("notified_at_utc")?;
    Ok(MaintenanceListItem {
        key,
        category_id: row.try_get("category_id")?,
        category: CategoryDisplay {
            id: row.try_get("category_id")?,
            key: row.try_get("category_display_key")?,
            icon: row.try_get("category_icon")?,
            color: row.try_get("category_color")?,
            label: row.try_get("category_label")?,
        },
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
  m.key,
  m.category_id::TEXT AS category_id,
  c.key AS category_display_key,
  c.icon AS category_icon,
  c.color AS category_color,
  coalesce((
    SELECT ci.label
    FROM event_category_i18n ci
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ci.locale
    WHERE ci.category_id = c.id
    ORDER BY lp.ord
    LIMIT 1
  ), c.key) AS category_label,
  m.start_utc,
  m.end_utc,
  m.notified_at_utc,
  m.last_modified_at,
  u.id AS last_modified_by_user_id,
  u.email AS last_modified_by_email,
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
JOIN event_categories c ON c.id = m.category_id
LEFT JOIN users u ON u.id = m.last_modified_by_user_id
WHERE m.key = $1
"#,
    )
    .bind(maintenance_code)
    .bind(locale_chain)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let key: String = row.try_get("key")?;
    let start_utc: DateTime<Utc> = row.try_get("start_utc")?;
    let end_utc: Option<DateTime<Utc>> = row.try_get("end_utc")?;
    let notified_at_utc: Option<DateTime<Utc>> = row.try_get("notified_at_utc")?;
    let last_modified_at: Option<DateTime<Utc>> = row.try_get("last_modified_at")?;
    let last_modified_by_user_id: Option<Uuid> = row.try_get("last_modified_by_user_id")?;
    let last_modified_by_email: Option<String> = row.try_get("last_modified_by_email")?;
    Ok(Some(MaintenanceDetail {
        key,
        category_id: row.try_get("category_id")?,
        category: CategoryDisplay {
            id: row.try_get("category_id")?,
            key: row.try_get("category_display_key")?,
            icon: row.try_get("category_icon")?,
            color: row.try_get("category_color")?,
            label: row.try_get("category_label")?,
        },
        title: row.try_get("title")?,
        warning: row.try_get("warning")?,
        short_description: row.try_get("short_description")?,
        long_description: row.try_get("long_description")?,
        location: row.try_get("location")?,
        start_utc: start_utc.to_rfc3339(),
        end_utc: end_utc.map(|value| value.to_rfc3339()),
        notified_at_utc: notified_at_utc.map(|value| value.to_rfc3339()),
        last_modified_at: last_modified_at.map(|value| value.to_rfc3339()),
        last_modified_by: last_modified_by_user_id.zip(last_modified_by_email).map(|(id, email)| AuditUser {
            id: id.to_string(),
            email,
        }),
    }))
}

pub async fn edit_data(
    db: &sqlx::PgPool,
    maintenance_code: &str,
    locale: &str,
    locale_chain: &[String],
    enabled_locales: Vec<String>,
) -> Result<Option<MaintenanceEditData>, AppError> {
    let row = sqlx::query(
        "SELECT id, key, category_id::TEXT AS category_id, start_utc, end_utc, notified_at_utc FROM maintenances WHERE key = $1",
    )
    .bind(maintenance_code)
    .fetch_optional(db)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let maintenance_id: Uuid = row.try_get("id")?;
    let start_utc: DateTime<Utc> = row.try_get("start_utc")?;
    let end_utc: Option<DateTime<Utc>> = row.try_get("end_utc")?;
    let notified_at_utc: Option<DateTime<Utc>> = row.try_get("notified_at_utc")?;
    Ok(Some(MaintenanceEditData {
        key: row.try_get("key")?,
        category_id: row.try_get("category_id")?,
        start_utc: start_utc.to_rfc3339(),
        end_utc: end_utc.map(|value| value.to_rfc3339()),
        notified_at_utc: notified_at_utc.map(|value| value.to_rfc3339()),
        locale: locale.to_string(),
        enabled_locales,
        fields: edit_fields(db, maintenance_id, locale, locale_chain, &MAINTENANCE_FIELDS).await?,
    }))
}

const MAINTENANCE_FIELDS: [&str; 5] = ["title", "warning", "short_description", "long_description", "location"];

async fn edit_fields(
    db: &sqlx::PgPool,
    maintenance_id: Uuid,
    locale: &str,
    locale_chain: &[String],
    field_keys: &[&str],
) -> Result<Vec<EditFieldValue>, AppError> {
    let mut result = Vec::with_capacity(field_keys.len());
    for field_key in field_keys {
        let exact: Option<String> = sqlx::query_scalar(
            "SELECT field_value FROM maintenance_i18n WHERE maintenance_id = $1 AND locale = $2 AND field_key = $3",
        )
        .bind(maintenance_id)
        .bind(locale)
        .bind(field_key)
        .fetch_optional(db)
        .await?;
        let fallback = if exact.is_none() {
            sqlx::query(
                r#"
SELECT mi.locale, mi.field_value
FROM maintenance_i18n mi
JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = mi.locale
WHERE mi.maintenance_id = $1 AND mi.field_key = $3
ORDER BY lp.ord
LIMIT 1
"#,
            )
            .bind(maintenance_id)
            .bind(locale_chain)
            .bind(field_key)
            .fetch_optional(db)
            .await?
            .map(|row| Ok::<_, AppError>((row.try_get::<String, _>("locale")?, row.try_get::<String, _>("field_value")?)))
            .transpose()?
        } else {
            None
        };
        let (fallback_locale, fallback_value) = fallback.map_or((None, None), |(locale, value)| (Some(locale), Some(value)));
        result.push(EditFieldValue {
            field_key: (*field_key).to_string(),
            value: exact.clone().or_else(|| fallback_value.clone()).unwrap_or_default(),
            exact_value: exact,
            fallback_locale,
            fallback_value,
        });
    }
    Ok(result)
}

pub async fn save_partial(
    db: &sqlx::PgPool,
    payload: &MaintenanceSaveRequest,
    user_id: Uuid,
) -> Result<String, AppError> {
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
    if end_utc.is_some_and(|end| end < start_utc) {
        return Err(AppError::bad_request("maintenance end_utc cannot be earlier than start_utc"));
    }
    let notified_at_utc = payload
        .notified_at_utc
        .as_deref()
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|_| AppError::bad_request("invalid maintenance notified_at_utc"))?
        .map(|value| value.with_timezone(&Utc));

    let mut tx = db.begin().await?;
    let (maintenance_id, key): (Uuid, String) = if let Some(key) = &payload.key {
        let row = sqlx::query(
            r#"
UPDATE maintenances
SET category_id = $2::uuid,
    start_utc = $3,
    end_utc = $4,
    notified_at_utc = $5,
    updated_at = now(),
    last_modified_at = now(),
    last_modified_by_user_id = $6
WHERE key = $1
RETURNING id, key
"#,
        )
        .bind(key)
        .bind(&payload.category_id)
        .bind(start_utc)
        .bind(end_utc)
        .bind(notified_at_utc)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("maintenance not found"))?;
        (row.try_get("id")?, row.try_get("key")?)
    } else {
        let row = sqlx::query(
            r#"
INSERT INTO maintenances (id, key, category_id, start_utc, end_utc, notified_at_utc, updated_at, last_modified_at, last_modified_by_user_id)
VALUES (gen_random_uuid(), 'EVT-' || nextval('maintenance_key_seq'), $1::uuid, $2, $3, $4, now(), now(), $5)
RETURNING id, key
"#,
        )
        .bind(&payload.category_id)
        .bind(start_utc)
        .bind(end_utc)
        .bind(notified_at_utc)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;
        (row.try_get("id")?, row.try_get("key")?)
    };

    for (field_key, field_value) in &payload.fields {
        let trimmed = field_value.trim();
        if trimmed.is_empty() {
            sqlx::query("DELETE FROM maintenance_i18n WHERE maintenance_id = $1 AND locale = $2 AND field_key = $3")
                .bind(maintenance_id)
                .bind(&payload.locale)
                .bind(field_key)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query(
                r#"
INSERT INTO maintenance_i18n (maintenance_id, locale, field_key, field_value)
VALUES ($1, $2, $3, $4)
ON CONFLICT (maintenance_id, locale, field_key) DO UPDATE SET field_value = EXCLUDED.field_value
"#,
            )
            .bind(maintenance_id)
            .bind(&payload.locale)
            .bind(field_key)
            .bind(trimmed)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;
    Ok(key)
}

pub async fn list_translations(
    db: &sqlx::PgPool,
    maintenance_code: &str,
) -> Result<Vec<MaintenanceTranslationMatrixRow>, AppError> {
    let maintenance_exists: Option<bool> = sqlx::query_scalar("SELECT TRUE FROM maintenances WHERE key = $1")
        .bind(maintenance_code)
        .fetch_optional(db)
        .await?;
    if maintenance_exists.is_none() {
        return Err(AppError::not_found("maintenance not found"));
    }

    let rows = sqlx::query(
        r#"
WITH target AS (
  SELECT id FROM maintenances WHERE key = $1
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
    let maintenance_id: Uuid = sqlx::query_scalar("SELECT id FROM maintenances WHERE key = $1")
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

pub async fn delete_by_code(db: &sqlx::PgPool, maintenance_code: &str) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM maintenances WHERE key = $1")
        .bind(maintenance_code)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
}
