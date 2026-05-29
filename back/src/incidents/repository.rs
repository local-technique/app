use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::incidents::model::{
    IncidentDetail, IncidentListItem, IncidentTimelineItem, IncidentTimelineUpsertItem,
    IncidentTranslationMatrixRow, IncidentTranslationValue, IncidentUpsertRequest,
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
  i.code,
  i.category_code,
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
  ), '') AS location
FROM incidents i
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
  OR i.category_code ILIKE ('%' || $2 || '%')
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
    let id: String = row.try_get("code")?;
    let start_utc: DateTime<Utc> = row.try_get("start_utc")?;
    let end_utc: Option<DateTime<Utc>> = row.try_get("end_utc")?;
    Ok(IncidentListItem {
        id,
        category_code: row.try_get("category_code")?,
        title: row.try_get("title")?,
        short_description: row.try_get("short_description")?,
        location: row.try_get("location")?,
        start_utc: start_utc.to_rfc3339(),
        end_utc: end_utc.map(|value| value.to_rfc3339()),
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
  i.code,
  i.category_code,
  i.start_utc,
  i.end_utc,
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
WHERE i.code = $1
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
ORDER BY t.sort_order ASC, t.at_utc ASC
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
            let at_utc: DateTime<Utc> = value.try_get("at_utc")?;
            Ok(IncidentTimelineItem {
                id: id.to_string(),
                at_utc: at_utc.to_rfc3339(),
                title: value.try_get("title")?,
                details: value.try_get("details")?,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    let code: String = row.try_get("code")?;
    let start_utc: DateTime<Utc> = row.try_get("start_utc")?;
    let end_utc: Option<DateTime<Utc>> = row.try_get("end_utc")?;
    Ok(Some(IncidentDetail {
        id: code,
        category_code: row.try_get("category_code")?,
        title: row.try_get("title")?,
        short_description: row.try_get("short_description")?,
        long_description: row.try_get("long_description")?,
        location: row.try_get("location")?,
        start_utc: start_utc.to_rfc3339(),
        end_utc: end_utc.map(|value| value.to_rfc3339()),
        timeline,
    }))
}

pub async fn list_translations(
    db: &sqlx::PgPool,
    incident_code: &str,
) -> Result<Vec<IncidentTranslationMatrixRow>, AppError> {
    let incident_exists: Option<bool> = sqlx::query_scalar("SELECT TRUE FROM incidents WHERE code = $1")
        .bind(incident_code)
        .fetch_optional(db)
        .await?;
    if incident_exists.is_none() {
        return Err(AppError::not_found("incident not found"));
    }

    let rows = sqlx::query(
        r#"
WITH target AS (
  SELECT id FROM incidents WHERE code = $1
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
    let incident_id: Uuid = sqlx::query_scalar("SELECT id FROM incidents WHERE code = $1")
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

pub async fn upsert(db: &sqlx::PgPool, payload: &IncidentUpsertRequest) -> Result<(), AppError> {
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

    let mut tx = db.begin().await?;

    let incident_id: Uuid = sqlx::query_scalar::<_, Uuid>(
        r#"
INSERT INTO incidents (id, code, category_code, start_utc, end_utc, updated_at)
VALUES ($1, $2, $3, $4, $5, now())
ON CONFLICT (code) DO UPDATE
SET category_code = EXCLUDED.category_code,
    start_utc = EXCLUDED.start_utc,
    end_utc = EXCLUDED.end_utc,
    updated_at = now()
RETURNING id
"#,
    )
    .bind(Uuid::new_v4())
    .bind(payload.id.as_str())
    .bind(payload.category_code.as_str())
    .bind(start_utc)
    .bind(end_utc)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM incident_i18n WHERE incident_id = $1")
        .bind(incident_id)
        .execute(&mut *tx)
        .await?;

    for value in &payload.translations {
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

    sqlx::query("DELETE FROM incident_timeline_i18n WHERE timeline_id IN (SELECT id FROM incident_timeline WHERE incident_id = $1)")
        .bind(incident_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM incident_timeline WHERE incident_id = $1")
        .bind(incident_id)
        .execute(&mut *tx)
        .await?;

    for item in &payload.timeline {
        upsert_timeline_item(&mut tx, incident_id, item).await?;
    }

    tx.commit().await?;
    Ok(())
}

async fn upsert_timeline_item(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    incident_id: Uuid,
    item: &IncidentTimelineUpsertItem,
) -> Result<(), AppError> {
    let timeline_id = Uuid::parse_str(&item.id).map_err(|_| AppError::bad_request("invalid timeline id"))?;
    let at_utc = DateTime::parse_from_rfc3339(&item.at_utc)
        .map_err(|_| AppError::bad_request("invalid timeline at_utc"))?
        .with_timezone(&Utc);

    sqlx::query("INSERT INTO incident_timeline (id, incident_id, at_utc, sort_order) VALUES ($1, $2, $3, $4)")
        .bind(timeline_id)
        .bind(incident_id)
        .bind(at_utc)
        .bind(item.sort_order)
        .execute(&mut **tx)
        .await?;

    for value in &item.translations {
        sqlx::query(
            "INSERT INTO incident_timeline_i18n (timeline_id, locale, field_key, field_value) VALUES ($1, $2, $3, $4)",
        )
        .bind(timeline_id)
        .bind(value.locale.trim().to_lowercase())
        .bind(value.field_key.trim().to_lowercase())
        .bind(value.field_value.trim())
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

pub async fn delete_by_code(db: &sqlx::PgPool, incident_code: &str) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM incidents WHERE code = $1")
        .bind(incident_code)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
}
