use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::maintenances::model::{
    AuditUser, CategoryDisplay, EditFieldValue, MaintenanceDetail, MaintenanceEditData, MaintenanceListItem,
    MaintenanceSaveRequest, MaintenanceTimelineEditItem, MaintenanceTimelineItem,
    MaintenanceTranslationMatrixRow, MaintenanceTranslationValue,
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
  m.status_type,
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
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'description'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS description,
  coalesce((
    SELECT mi.field_value
    FROM maintenance_i18n mi
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = mi.locale
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'location'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS location,
  coalesce((
    SELECT mi.field_value
    FROM maintenance_i18n mi
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = mi.locale
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'status_text'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS status_text,
  lt.id AS latest_timeline_id,
  lt.at_utc AS latest_timeline_at_utc,
  coalesce((
    SELECT ti.field_value
    FROM maintenance_timeline_i18n ti
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale
    WHERE ti.timeline_id = lt.id AND ti.field_key = 'title'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS latest_timeline_title,
  coalesce((
    SELECT ti.field_value
    FROM maintenance_timeline_i18n ti
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale
    WHERE ti.timeline_id = lt.id AND ti.field_key = 'details'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS latest_timeline_details
FROM maintenances m
LEFT JOIN LATERAL (
  SELECT t.id, t.at_utc
  FROM maintenance_timeline t
  WHERE t.maintenance_id = m.id
  ORDER BY t.at_utc DESC NULLS FIRST, t.sort_order ASC
  LIMIT 1
) lt ON TRUE
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
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'description'
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
    let latest_timeline_id: Option<Uuid> = row.try_get("latest_timeline_id")?;
    let latest_timeline_at_utc: Option<DateTime<Utc>> = row.try_get("latest_timeline_at_utc")?;
    let timeline = latest_timeline_id.map_or_else(Vec::new, |id| {
        vec![MaintenanceTimelineItem {
            id: id.to_string(),
            at_utc: latest_timeline_at_utc.map(|value| value.to_rfc3339()),
            title: row.try_get("latest_timeline_title").unwrap_or_default(),
            details: row.try_get("latest_timeline_details").unwrap_or_default(),
        }]
    });
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
        description: row.try_get("description")?,
        location: row.try_get("location")?,
        start_utc: start_utc.to_rfc3339(),
        end_utc: end_utc.map(|value| value.to_rfc3339()),
        status_type: row.try_get("status_type")?,
        status_text: row.try_get("status_text")?,
        timeline,
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
  m.status_type,
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
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'description'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS description,
  coalesce((
    SELECT mi.field_value
    FROM maintenance_i18n mi
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = mi.locale
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'location'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS location,
  coalesce((
    SELECT mi.field_value
    FROM maintenance_i18n mi
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = mi.locale
    WHERE mi.maintenance_id = m.id AND mi.field_key = 'status_text'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS status_text
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

    let maintenance_id: Uuid = row.try_get("id")?;
    let timeline_rows = sqlx::query(
        r#"
SELECT
  t.id,
  t.at_utc,
  coalesce((
    SELECT ti.field_value
    FROM maintenance_timeline_i18n ti
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale
    WHERE ti.timeline_id = t.id AND ti.field_key = 'title'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS title,
  coalesce((
    SELECT ti.field_value
    FROM maintenance_timeline_i18n ti
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale
    WHERE ti.timeline_id = t.id AND ti.field_key = 'details'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS details
FROM maintenance_timeline t
WHERE t.maintenance_id = $1
ORDER BY t.at_utc DESC NULLS FIRST, t.sort_order ASC
"#,
    )
    .bind(maintenance_id)
    .bind(locale_chain)
    .fetch_all(db)
    .await?;

    let timeline = timeline_rows
        .into_iter()
        .map(|value| {
            let id: Uuid = value.try_get("id")?;
            let at_utc: Option<DateTime<Utc>> = value.try_get("at_utc")?;
            Ok(MaintenanceTimelineItem {
                id: id.to_string(),
                at_utc: at_utc.map(|value| value.to_rfc3339()),
                title: value.try_get("title")?,
                details: value.try_get("details")?,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    let key: String = row.try_get("key")?;
    let start_utc: DateTime<Utc> = row.try_get("start_utc")?;
    let end_utc: Option<DateTime<Utc>> = row.try_get("end_utc")?;
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
        description: row.try_get("description")?,
        location: row.try_get("location")?,
        start_utc: start_utc.to_rfc3339(),
        end_utc: end_utc.map(|value| value.to_rfc3339()),
        status_type: row.try_get("status_type")?,
        status_text: row.try_get("status_text")?,
        timeline,
        last_modified_at: last_modified_at.map(|value| value.to_rfc3339()),
        last_modified_by: last_modified_by_user_id.zip(last_modified_by_email).map(|(id, email)| AuditUser {
            id: id.to_string(),
            email,
        }),
    }))
}

const MAINTENANCE_TIMELINE_FIELDS: [&str; 2] = ["title", "details"];

pub async fn edit_data(
    db: &sqlx::PgPool,
    maintenance_code: &str,
    locale: &str,
    locale_chain: &[String],
    enabled_locales: Vec<String>,
) -> Result<Option<MaintenanceEditData>, AppError> {
    let row = sqlx::query(
        "SELECT id, key, category_id::TEXT AS category_id, start_utc, end_utc, status_type FROM maintenances WHERE key = $1",
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
    let timeline_rows = sqlx::query(
        "SELECT id, at_utc, sort_order FROM maintenance_timeline WHERE maintenance_id = $1 ORDER BY at_utc DESC NULLS FIRST, sort_order ASC",
    )
    .bind(maintenance_id)
    .fetch_all(db)
    .await?;
    let mut timeline = Vec::with_capacity(timeline_rows.len());
    for timeline_row in timeline_rows {
        let timeline_id: Uuid = timeline_row.try_get("id")?;
        let at_utc: Option<DateTime<Utc>> = timeline_row.try_get("at_utc")?;
        timeline.push(MaintenanceTimelineEditItem {
            id: timeline_id.to_string(),
            at_utc: at_utc.map(|value| value.to_rfc3339()),
            sort_order: timeline_row.try_get("sort_order")?,
            fields: crate::common::timeline::edit_timeline_fields(db, "maintenance_timeline_i18n", timeline_id, locale, locale_chain, &MAINTENANCE_TIMELINE_FIELDS).await?,
        });
    }
    Ok(Some(MaintenanceEditData {
        key: row.try_get("key")?,
        category_id: row.try_get("category_id")?,
        start_utc: start_utc.to_rfc3339(),
        end_utc: end_utc.map(|value| value.to_rfc3339()),
        status_type: row.try_get("status_type")?,
        locale: locale.to_string(),
        enabled_locales,
        fields: edit_fields(db, maintenance_id, locale, locale_chain, &MAINTENANCE_FIELDS).await?,
        timeline,
    }))
}

const MAINTENANCE_FIELDS: [&str; 5] = ["title", "warning", "description", "location", "status_text"];

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

    let mut tx = db.begin().await?;
    let (maintenance_id, key): (Uuid, String) = if let Some(key) = &payload.key {
        let row = sqlx::query(
            r#"
UPDATE maintenances
SET category_id = $2::uuid,
    start_utc = $3,
    end_utc = $4,
    status_type = $5,
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
        .bind(&payload.status_type)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("maintenance not found"))?;
        (row.try_get("id")?, row.try_get("key")?)
    } else {
        let row = sqlx::query(
            r#"
INSERT INTO maintenances (id, key, category_id, start_utc, end_utc, status_type, updated_at, last_modified_at, last_modified_by_user_id)
VALUES (gen_random_uuid(), 'EVT-' || nextval('maintenance_key_seq'), $1::uuid, $2, $3, $4, now(), now(), $5)
RETURNING id, key
"#,
        )
        .bind(&payload.category_id)
        .bind(start_utc)
        .bind(end_utc)
        .bind(&payload.status_type)
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

    let submitted_ids = payload
        .timeline
        .iter()
        .map(|item| Uuid::parse_str(&item.id).map_err(|_| AppError::bad_request("invalid timeline id")))
        .collect::<Result<Vec<_>, AppError>>()?;
    if !submitted_ids.is_empty() {
        let foreign_timeline_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM maintenance_timeline WHERE id = ANY($1) AND maintenance_id <> $2 LIMIT 1",
        )
        .bind(&submitted_ids)
        .bind(maintenance_id)
        .fetch_optional(&mut *tx)
        .await?;
        if foreign_timeline_id.is_some() {
            return Err(AppError::bad_request("timeline id does not belong to maintenance"));
        }
    }
    if payload.replace_timeline && !submitted_ids.is_empty() {
        sqlx::query("DELETE FROM maintenance_timeline WHERE maintenance_id = $1 AND NOT (id = ANY($2))")
            .bind(maintenance_id)
            .bind(&submitted_ids)
            .execute(&mut *tx)
            .await?;
    }
    for item in &payload.timeline {
        let timeline_id = Uuid::parse_str(&item.id).map_err(|_| AppError::bad_request("invalid timeline id"))?;
        let at_utc = item
            .at_utc
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(DateTime::parse_from_rfc3339)
            .transpose()
            .map_err(|_| AppError::bad_request("invalid timeline at_utc"))?
            .map(|value| value.with_timezone(&Utc));
        sqlx::query(
            r#"
INSERT INTO maintenance_timeline (id, maintenance_id, at_utc, sort_order)
VALUES ($1, $2, $3, $4)
ON CONFLICT (id) DO UPDATE SET at_utc = EXCLUDED.at_utc, sort_order = EXCLUDED.sort_order
WHERE maintenance_timeline.maintenance_id = EXCLUDED.maintenance_id
"#,
        )
        .bind(timeline_id)
        .bind(maintenance_id)
        .bind(at_utc)
        .bind(item.sort_order)
        .execute(&mut *tx)
        .await?;
        for (field_key, field_value) in &item.fields {
            let trimmed = field_value.trim();
            if trimmed.is_empty() {
                sqlx::query("DELETE FROM maintenance_timeline_i18n WHERE timeline_id = $1 AND locale = $2 AND field_key = $3")
                    .bind(timeline_id)
                    .bind(&payload.locale)
                    .bind(field_key)
                    .execute(&mut *tx)
                    .await?;
            } else {
                sqlx::query(
                    r#"
INSERT INTO maintenance_timeline_i18n (timeline_id, locale, field_key, field_value)
VALUES ($1, $2, $3, $4)
ON CONFLICT (timeline_id, locale, field_key) DO UPDATE SET field_value = EXCLUDED.field_value
"#,
                )
                .bind(timeline_id)
                .bind(&payload.locale)
                .bind(field_key)
                .bind(trimmed)
                .execute(&mut *tx)
                .await?;
            }
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

async fn save_timeline_field(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    timeline_id: Uuid,
    locale: &str,
    field_key: &str,
    field_value: &str,
) -> Result<(), AppError> {
    let trimmed = field_value.trim();
    if trimmed.is_empty() {
        sqlx::query("DELETE FROM maintenance_timeline_i18n WHERE timeline_id = $1 AND locale = $2 AND field_key = $3")
            .bind(timeline_id)
            .bind(locale)
            .bind(field_key)
            .execute(&mut **tx)
            .await?;
    } else {
        sqlx::query(
            r#"
INSERT INTO maintenance_timeline_i18n (timeline_id, locale, field_key, field_value)
VALUES ($1, $2, $3, $4)
ON CONFLICT (timeline_id, locale, field_key) DO UPDATE SET field_value = EXCLUDED.field_value
"#,
        )
        .bind(timeline_id)
        .bind(locale)
        .bind(field_key)
        .bind(trimmed)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

pub async fn create_timeline_entry(
    db: &sqlx::PgPool,
    maintenance_code: &str,
    at_utc: Option<DateTime<Utc>>,
    sort_order: i32,
    locale: &str,
    fields: &HashMap<String, String>,
) -> Result<MaintenanceTimelineItem, AppError> {
    let maintenance_id: Uuid = sqlx::query_scalar("SELECT id FROM maintenances WHERE key = $1")
        .bind(maintenance_code)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::not_found("maintenance not found"))?;

    let mut tx = db.begin().await?;
    let timeline_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO maintenance_timeline (id, maintenance_id, at_utc, sort_order) VALUES ($1, $2, $3, $4)",
    )
    .bind(timeline_id)
    .bind(maintenance_id)
    .bind(at_utc)
    .bind(sort_order)
    .execute(&mut *tx)
    .await?;

    for (field_key, field_value) in fields {
        save_timeline_field(&mut tx, timeline_id, locale, field_key, field_value).await?;
    }
    tx.commit().await?;

    let locale_chain = crate::common::i18n::locale_chain(Some(locale));
    let title = sqlx::query_scalar::<_, String>(
        "SELECT coalesce((SELECT ti.field_value FROM maintenance_timeline_i18n ti JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale WHERE ti.timeline_id = $1 AND ti.field_key = 'title' ORDER BY lp.ord LIMIT 1), '')",
    )
    .bind(timeline_id)
    .bind(&locale_chain)
    .fetch_one(db)
    .await?;

    let details = sqlx::query_scalar::<_, String>(
        "SELECT coalesce((SELECT ti.field_value FROM maintenance_timeline_i18n ti JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale WHERE ti.timeline_id = $1 AND ti.field_key = 'details' ORDER BY lp.ord LIMIT 1), '')",
    )
    .bind(timeline_id)
    .bind(&locale_chain)
    .fetch_one(db)
    .await?;

    Ok(MaintenanceTimelineItem {
        id: timeline_id.to_string(),
        at_utc: at_utc.map(|v| v.to_rfc3339()),
        title,
        details,
    })
}

pub async fn update_timeline_entry(
    db: &sqlx::PgPool,
    maintenance_code: &str,
    entry_id: &str,
    at_utc: Option<DateTime<Utc>>,
    sort_order: i32,
    locale: &str,
    fields: &HashMap<String, String>,
) -> Result<Option<MaintenanceTimelineItem>, AppError> {
    let maintenance_id: Uuid = sqlx::query_scalar("SELECT id FROM maintenances WHERE key = $1")
        .bind(maintenance_code)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::not_found("maintenance not found"))?;

    let timeline_id = Uuid::parse_str(entry_id).map_err(|_| AppError::bad_request("invalid timeline id"))?;

    let mut tx = db.begin().await?;
    let updated = sqlx::query(
        "UPDATE maintenance_timeline SET at_utc = $1, sort_order = $4 WHERE id = $2 AND maintenance_id = $3",
    )
    .bind(at_utc)
    .bind(timeline_id)
    .bind(maintenance_id)
    .bind(sort_order)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if updated == 0 {
        return Ok(None);
    }

    for (field_key, field_value) in fields {
        save_timeline_field(&mut tx, timeline_id, locale, field_key, field_value).await?;
    }
    tx.commit().await?;

    let locale_chain = crate::common::i18n::locale_chain(Some(locale));
    let title = sqlx::query_scalar::<_, String>(
        "SELECT coalesce((SELECT ti.field_value FROM maintenance_timeline_i18n ti JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale WHERE ti.timeline_id = $1 AND ti.field_key = 'title' ORDER BY lp.ord LIMIT 1), '')",
    )
    .bind(timeline_id)
    .bind(&locale_chain)
    .fetch_one(db)
    .await?;

    let details = sqlx::query_scalar::<_, String>(
        "SELECT coalesce((SELECT ti.field_value FROM maintenance_timeline_i18n ti JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale WHERE ti.timeline_id = $1 AND ti.field_key = 'details' ORDER BY lp.ord LIMIT 1), '')",
    )
    .bind(timeline_id)
    .bind(&locale_chain)
    .fetch_one(db)
    .await?;

    Ok(Some(MaintenanceTimelineItem {
        id: timeline_id.to_string(),
        at_utc: at_utc.map(|v| v.to_rfc3339()),
        title,
        details,
    }))
}

pub async fn delete_timeline_entry(
    db: &sqlx::PgPool,
    maintenance_code: &str,
    entry_id: &str,
) -> Result<bool, AppError> {
    let maintenance_id: Uuid = sqlx::query_scalar("SELECT id FROM maintenances WHERE key = $1")
        .bind(maintenance_code)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::not_found("maintenance not found"))?;

    let timeline_id = Uuid::parse_str(entry_id).map_err(|_| AppError::bad_request("invalid timeline id"))?;
    let result = sqlx::query("DELETE FROM maintenance_timeline WHERE id = $1 AND maintenance_id = $2")
        .bind(timeline_id)
        .bind(maintenance_id)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
}
