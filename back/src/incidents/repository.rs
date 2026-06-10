use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::incidents::model::{
    AuditUser, CategoryDisplay, EditFieldValue, IncidentDetail, IncidentEditData, IncidentListItem,
    IncidentSaveRequest, IncidentTimelineEditItem, IncidentTimelineItem, IncidentTranslationMatrixRow,
    IncidentTranslationValue,
};

pub async fn list(
    db: &sqlx::PgPool,
    locale_chain: &[String],
    query: Option<&str>,
) -> Result<Vec<IncidentListItem>, AppError> {
    let rows = sqlx::query(
        r#"
SELECT
  i.id,
  i.key,
  i.category_id::TEXT AS category_id,
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
  i.start_utc,
  i.end_utc,
  coalesce((
    SELECT ii.field_value
    FROM incident_i18n ii
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ii.locale
    WHERE ii.incident_id = i.id AND ii.field_key = 'title'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS title,
  coalesce((
    SELECT ii.field_value
    FROM incident_i18n ii
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ii.locale
    WHERE ii.incident_id = i.id AND ii.field_key = 'short_description'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS short_description,
  coalesce((
    SELECT ii.field_value
    FROM incident_i18n ii
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ii.locale
    WHERE ii.incident_id = i.id AND ii.field_key = 'location'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS location,
  lt.id AS latest_timeline_id,
  lt.at_utc AS latest_timeline_at_utc,
  coalesce((
    SELECT ti.field_value
    FROM incident_timeline_i18n ti
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale
    WHERE ti.timeline_id = lt.id AND ti.field_key = 'title'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS latest_timeline_title,
  coalesce((
    SELECT ti.field_value
    FROM incident_timeline_i18n ti
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale
    WHERE ti.timeline_id = lt.id AND ti.field_key = 'details'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS latest_timeline_details
FROM incidents i
JOIN event_categories c ON c.id = i.category_id
LEFT JOIN LATERAL (
  SELECT t.id, t.at_utc
  FROM incident_timeline t
  WHERE t.incident_id = i.id
  ORDER BY t.at_utc DESC NULLS FIRST, t.sort_order ASC
  LIMIT 1
) lt ON TRUE
WHERE ($2::TEXT IS NULL OR $2 = '') OR (
  coalesce((
    SELECT ii.field_value
    FROM incident_i18n ii
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ii.locale
    WHERE ii.incident_id = i.id AND ii.field_key = 'title'
    ORDER BY lp.ord
    LIMIT 1
  ), '') ILIKE ('%' || $2 || '%')
  OR coalesce((
    SELECT ii.field_value
    FROM incident_i18n ii
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ii.locale
    WHERE ii.incident_id = i.id AND ii.field_key = 'short_description'
    ORDER BY lp.ord
    LIMIT 1
  ), '') ILIKE ('%' || $2 || '%')
  OR i.key ILIKE ('%' || $2 || '%')
  OR c.key ILIKE ('%' || $2 || '%')
)
ORDER BY i.start_utc ASC
"#,
    )
    .bind(locale_chain)
    .bind(query)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(to_list_item).collect()
}

fn to_list_item(row: sqlx::postgres::PgRow) -> Result<IncidentListItem, AppError> {
    let key: String = row.try_get("key")?;
    let start_utc: DateTime<Utc> = row.try_get("start_utc")?;
    let end_utc: Option<DateTime<Utc>> = row.try_get("end_utc")?;
    let latest_timeline_id: Option<Uuid> = row.try_get("latest_timeline_id")?;
    let latest_timeline_at_utc: Option<DateTime<Utc>> = row.try_get("latest_timeline_at_utc")?;
    let timeline = latest_timeline_id.map_or_else(Vec::new, |id| {
        vec![IncidentTimelineItem {
            id: id.to_string(),
            at_utc: latest_timeline_at_utc.map(|value| value.to_rfc3339()),
            title: row.try_get("latest_timeline_title").unwrap_or_default(),
            details: row.try_get("latest_timeline_details").unwrap_or_default(),
        }]
    });
    Ok(IncidentListItem {
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
        short_description: row.try_get("short_description")?,
        location: row.try_get("location")?,
        start_utc: start_utc.to_rfc3339(),
        end_utc: end_utc.map(|value| value.to_rfc3339()),
        timeline,
    })
}

pub async fn by_id(
    db: &sqlx::PgPool,
    incident_code: &str,
    locale_chain: &[String],
) -> Result<Option<IncidentDetail>, AppError> {
    let row = sqlx::query(
        r#"
SELECT
  i.id,
  i.key,
  i.category_id::TEXT AS category_id,
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
  i.start_utc,
  i.end_utc,
  i.last_modified_at,
  u.id AS last_modified_by_user_id,
  u.email AS last_modified_by_email,
  coalesce((
    SELECT ii.field_value
    FROM incident_i18n ii
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ii.locale
    WHERE ii.incident_id = i.id AND ii.field_key = 'title'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS title,
  coalesce((
    SELECT ii.field_value
    FROM incident_i18n ii
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ii.locale
    WHERE ii.incident_id = i.id AND ii.field_key = 'short_description'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS short_description,
  coalesce((
    SELECT ii.field_value
    FROM incident_i18n ii
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ii.locale
    WHERE ii.incident_id = i.id AND ii.field_key = 'long_description'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS long_description,
  coalesce((
    SELECT ii.field_value
    FROM incident_i18n ii
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ii.locale
    WHERE ii.incident_id = i.id AND ii.field_key = 'location'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS location
FROM incidents i
JOIN event_categories c ON c.id = i.category_id
LEFT JOIN users u ON u.id = i.last_modified_by_user_id
WHERE i.key = $1
"#,
    )
    .bind(incident_code)
    .bind(locale_chain)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let incident_id: Uuid = row.try_get("id")?;
    let timeline_rows = sqlx::query(
        r#"
SELECT
  t.id,
  t.at_utc,
  coalesce((
    SELECT ti.field_value
    FROM incident_timeline_i18n ti
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale
    WHERE ti.timeline_id = t.id AND ti.field_key = 'title'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS title,
  coalesce((
    SELECT ti.field_value
    FROM incident_timeline_i18n ti
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale
    WHERE ti.timeline_id = t.id AND ti.field_key = 'details'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS details
FROM incident_timeline t
WHERE t.incident_id = $1
ORDER BY t.at_utc DESC NULLS FIRST, t.sort_order ASC
"#,
    )
    .bind(incident_id)
    .bind(locale_chain)
    .fetch_all(db)
    .await?;

    let timeline = timeline_rows
        .into_iter()
        .map(|value| {
            let id: Uuid = value.try_get("id")?;
            let at_utc: Option<DateTime<Utc>> = value.try_get("at_utc")?;
            Ok(IncidentTimelineItem {
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
    Ok(Some(IncidentDetail {
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
        short_description: row.try_get("short_description")?,
        long_description: row.try_get("long_description")?,
        location: row.try_get("location")?,
        start_utc: start_utc.to_rfc3339(),
        end_utc: end_utc.map(|value| value.to_rfc3339()),
        timeline,
        last_modified_at: last_modified_at.map(|value| value.to_rfc3339()),
        last_modified_by: last_modified_by_user_id.zip(last_modified_by_email).map(|(id, email)| AuditUser {
            id: id.to_string(),
            email,
        }),
    }))
}

pub async fn edit_data(
    db: &sqlx::PgPool,
    incident_code: &str,
    locale: &str,
    locale_chain: &[String],
    enabled_locales: Vec<String>,
) -> Result<Option<IncidentEditData>, AppError> {
    let row = sqlx::query("SELECT id, key, category_id::TEXT AS category_id, start_utc, end_utc FROM incidents WHERE key = $1")
        .bind(incident_code)
        .fetch_optional(db)
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let incident_id: Uuid = row.try_get("id")?;
    let start_utc: DateTime<Utc> = row.try_get("start_utc")?;
    let end_utc: Option<DateTime<Utc>> = row.try_get("end_utc")?;
    let timeline_rows = sqlx::query(
        "SELECT id, at_utc, sort_order FROM incident_timeline WHERE incident_id = $1 ORDER BY at_utc DESC NULLS FIRST, sort_order ASC",
    )
    .bind(incident_id)
    .fetch_all(db)
    .await?;
    let mut timeline = Vec::with_capacity(timeline_rows.len());
    for timeline_row in timeline_rows {
        let timeline_id: Uuid = timeline_row.try_get("id")?;
        let at_utc: Option<DateTime<Utc>> = timeline_row.try_get("at_utc")?;
        timeline.push(IncidentTimelineEditItem {
            id: timeline_id.to_string(),
            at_utc: at_utc.map(|value| value.to_rfc3339()),
            sort_order: timeline_row.try_get("sort_order")?,
            fields: edit_timeline_fields(db, timeline_id, locale, locale_chain, &INCIDENT_TIMELINE_FIELDS).await?,
        });
    }
    Ok(Some(IncidentEditData {
        key: row.try_get("key")?,
        category_id: row.try_get("category_id")?,
        start_utc: start_utc.to_rfc3339(),
        end_utc: end_utc.map(|value| value.to_rfc3339()),
        locale: locale.to_string(),
        enabled_locales,
        fields: edit_fields(db, incident_id, locale, locale_chain, &INCIDENT_FIELDS).await?,
        timeline,
    }))
}

const INCIDENT_FIELDS: [&str; 4] = ["title", "short_description", "long_description", "location"];
const INCIDENT_TIMELINE_FIELDS: [&str; 2] = ["title", "details"];

async fn edit_fields(
    db: &sqlx::PgPool,
    incident_id: Uuid,
    locale: &str,
    locale_chain: &[String],
    field_keys: &[&str],
) -> Result<Vec<EditFieldValue>, AppError> {
    let mut result = Vec::with_capacity(field_keys.len());
    for field_key in field_keys {
        let exact: Option<String> = sqlx::query_scalar(
            "SELECT field_value FROM incident_i18n WHERE incident_id = $1 AND locale = $2 AND field_key = $3",
        )
        .bind(incident_id)
        .bind(locale)
        .bind(field_key)
        .fetch_optional(db)
        .await?;
        let fallback = if exact.is_none() {
            sqlx::query(
                r#"
SELECT ii.locale, ii.field_value
FROM incident_i18n ii
JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ii.locale
WHERE ii.incident_id = $1 AND ii.field_key = $3
ORDER BY lp.ord
LIMIT 1
"#,
            )
            .bind(incident_id)
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

async fn edit_timeline_fields(
    db: &sqlx::PgPool,
    timeline_id: Uuid,
    locale: &str,
    locale_chain: &[String],
    field_keys: &[&str],
) -> Result<Vec<EditFieldValue>, AppError> {
    let mut result = Vec::with_capacity(field_keys.len());
    for field_key in field_keys {
        let exact: Option<String> = sqlx::query_scalar(
            "SELECT field_value FROM incident_timeline_i18n WHERE timeline_id = $1 AND locale = $2 AND field_key = $3",
        )
        .bind(timeline_id)
        .bind(locale)
        .bind(field_key)
        .fetch_optional(db)
        .await?;
        let fallback = if exact.is_none() {
            sqlx::query(
                r#"
SELECT ti.locale, ti.field_value
FROM incident_timeline_i18n ti
JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale
WHERE ti.timeline_id = $1 AND ti.field_key = $3
ORDER BY lp.ord
LIMIT 1
"#,
            )
            .bind(timeline_id)
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

pub async fn save_partial(db: &sqlx::PgPool, payload: &IncidentSaveRequest, user_id: Uuid) -> Result<String, AppError> {
    let start_utc = DateTime::parse_from_rfc3339(&payload.start_utc)
        .map_err(|_| AppError::bad_request("invalid incident start_utc"))?
        .with_timezone(&Utc);
    let end_utc = payload
        .end_utc
        .as_deref()
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|_| AppError::bad_request("invalid incident end_utc"))?
        .map(|value| value.with_timezone(&Utc));
    if end_utc.is_some_and(|end| end < start_utc) {
        return Err(AppError::bad_request("incident end_utc cannot be earlier than start_utc"));
    }

    let mut tx = db.begin().await?;
    let (incident_id, key): (Uuid, String) = if let Some(key) = &payload.key {
        let row = sqlx::query(
            r#"
UPDATE incidents
SET category_id = $2::uuid,
    start_utc = $3,
    end_utc = $4,
    updated_at = now(),
    last_modified_at = now(),
    last_modified_by_user_id = $5
WHERE key = $1
RETURNING id, key
"#,
        )
        .bind(key)
        .bind(&payload.category_id)
        .bind(start_utc)
        .bind(end_utc)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("incident not found"))?;
        (row.try_get("id")?, row.try_get("key")?)
    } else {
        let row = sqlx::query(
            r#"
INSERT INTO incidents (id, key, category_id, start_utc, end_utc, updated_at, last_modified_at, last_modified_by_user_id)
VALUES (gen_random_uuid(), 'INC-' || nextval('incident_key_seq'), $1::uuid, $2, $3, now(), now(), $4)
RETURNING id, key
"#,
        )
        .bind(&payload.category_id)
        .bind(start_utc)
        .bind(end_utc)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;
        (row.try_get("id")?, row.try_get("key")?)
    };

    for (field_key, field_value) in &payload.fields {
        save_field(&mut tx, "incident_i18n", "incident_id", incident_id, &payload.locale, field_key, field_value).await?;
    }

    let submitted_ids = payload
        .timeline
        .iter()
        .map(|item| Uuid::parse_str(&item.id).map_err(|_| AppError::bad_request("invalid timeline id")))
        .collect::<Result<Vec<_>, AppError>>()?;
    if !submitted_ids.is_empty() {
        let foreign_timeline_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM incident_timeline WHERE id = ANY($1) AND incident_id <> $2 LIMIT 1",
        )
        .bind(&submitted_ids)
        .bind(incident_id)
        .fetch_optional(&mut *tx)
        .await?;
        if foreign_timeline_id.is_some() {
            return Err(AppError::bad_request("timeline id does not belong to incident"));
        }
    }
    if payload.replace_timeline {
        sqlx::query("DELETE FROM incident_timeline WHERE incident_id = $1 AND NOT (id = ANY($2))")
            .bind(incident_id)
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
INSERT INTO incident_timeline (id, incident_id, at_utc, sort_order)
VALUES ($1, $2, $3, $4)
ON CONFLICT (id) DO UPDATE SET at_utc = EXCLUDED.at_utc, sort_order = EXCLUDED.sort_order
WHERE incident_timeline.incident_id = EXCLUDED.incident_id
"#,
        )
        .bind(timeline_id)
        .bind(incident_id)
        .bind(at_utc)
        .bind(item.sort_order)
        .execute(&mut *tx)
        .await?;
        for (field_key, field_value) in &item.fields {
            save_timeline_field(&mut tx, timeline_id, &payload.locale, field_key, field_value).await?;
        }
    }
    tx.commit().await?;
    Ok(key)
}

async fn save_field(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    table: &str,
    id_column: &str,
    owner_id: Uuid,
    locale: &str,
    field_key: &str,
    field_value: &str,
) -> Result<(), AppError> {
    let trimmed = field_value.trim();
    if trimmed.is_empty() {
        let sql = format!("DELETE FROM {table} WHERE {id_column} = $1 AND locale = $2 AND field_key = $3");
        sqlx::query(&sql)
            .bind(owner_id)
            .bind(locale)
            .bind(field_key)
            .execute(&mut **tx)
            .await?;
    } else {
        let sql = format!(
            "INSERT INTO {table} ({id_column}, locale, field_key, field_value) VALUES ($1, $2, $3, $4) ON CONFLICT ({id_column}, locale, field_key) DO UPDATE SET field_value = EXCLUDED.field_value"
        );
        sqlx::query(&sql)
            .bind(owner_id)
            .bind(locale)
            .bind(field_key)
            .bind(trimmed)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
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
        sqlx::query("DELETE FROM incident_timeline_i18n WHERE timeline_id = $1 AND locale = $2 AND field_key = $3")
            .bind(timeline_id)
            .bind(locale)
            .bind(field_key)
            .execute(&mut **tx)
            .await?;
    } else {
        sqlx::query(
            r#"
INSERT INTO incident_timeline_i18n (timeline_id, locale, field_key, field_value)
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

pub async fn list_translations(
    db: &sqlx::PgPool,
    incident_code: &str,
) -> Result<Vec<IncidentTranslationMatrixRow>, AppError> {
    let incident_exists: Option<bool> = sqlx::query_scalar("SELECT TRUE FROM incidents WHERE key = $1")
        .bind(incident_code)
        .fetch_optional(db)
        .await?;
    if incident_exists.is_none() {
        return Err(AppError::not_found("incident not found"));
    }

    let rows = sqlx::query(
        r#"
WITH target AS (
  SELECT id FROM incidents WHERE key = $1
)
SELECT
  k.field_key,
  l.code AS locale,
  ii.field_value
FROM locales l
CROSS JOIN (
  SELECT DISTINCT field_key
  FROM incident_i18n
  WHERE incident_id = (SELECT id FROM target)
) k
LEFT JOIN incident_i18n ii
  ON ii.incident_id = (SELECT id FROM target)
 AND ii.locale = l.code
 AND ii.field_key = k.field_key
ORDER BY k.field_key, l.code
"#,
    )
    .bind(incident_code)
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(IncidentTranslationMatrixRow {
                field_key: row.try_get("field_key")?,
                locale: row.try_get("locale")?,
                field_value: row.try_get("field_value")?,
            })
        })
        .collect()
}

pub async fn replace_translations(
    db: &sqlx::PgPool,
    incident_code: &str,
    values: &[IncidentTranslationValue],
) -> Result<(), AppError> {
    let incident_id: Uuid = sqlx::query_scalar("SELECT id FROM incidents WHERE key = $1")
        .bind(incident_code)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::not_found("incident not found"))?;

    let mut tx = db.begin().await?;

    sqlx::query("DELETE FROM incident_i18n WHERE incident_id = $1")
        .bind(incident_id)
        .execute(&mut *tx)
        .await?;

    for value in values {
        sqlx::query(
            "INSERT INTO incident_i18n (incident_id, locale, field_key, field_value) VALUES ($1, $2, $3, $4)",
        )
        .bind(incident_id)
        .bind(value.locale.trim().to_lowercase())
        .bind(value.field_key.trim().to_lowercase())
        .bind(value.field_value.trim())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn delete_by_code(db: &sqlx::PgPool, incident_code: &str) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM incidents WHERE key = $1")
        .bind(incident_code)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
}
