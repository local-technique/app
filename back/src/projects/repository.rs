use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::projects::model::{
    AuditUser, CategoryDisplay, EditFieldValue, ProjectDetail, ProjectEditData, ProjectListItem,
    ProjectSaveRequest, ProjectTimelineEditItem, ProjectTimelineItem, ProjectTranslationMatrixRow,
    ProjectTranslationValue,
};

pub async fn list(
    db: &sqlx::PgPool,
    locale_chain: &[String],
    query: Option<&str>,
) -> Result<Vec<ProjectListItem>, AppError> {
    let rows = sqlx::query(
        r#"
SELECT
  p.key,
  p.category_id::TEXT AS category_id,
  p.start_utc,
  p.end_utc,
  p.status_type,
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
  coalesce((
    SELECT pi.field_value
    FROM project_i18n pi
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = pi.locale
    WHERE pi.project_id = p.id AND pi.field_key = 'title'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS title,
  coalesce((
    SELECT pi.field_value
    FROM project_i18n pi
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = pi.locale
    WHERE pi.project_id = p.id AND pi.field_key = 'description'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS description
  , coalesce((
    SELECT pi.field_value
    FROM project_i18n pi
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = pi.locale
    WHERE pi.project_id = p.id AND pi.field_key = 'status_text'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS status_text,
  lt.id AS latest_timeline_id,
  lt.at_utc AS latest_timeline_at_utc,
  coalesce((
    SELECT ti.field_value
    FROM project_timeline_i18n ti
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale
    WHERE ti.timeline_id = lt.id AND ti.field_key = 'title'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS latest_timeline_title,
  coalesce((
    SELECT ti.field_value
    FROM project_timeline_i18n ti
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale
    WHERE ti.timeline_id = lt.id AND ti.field_key = 'details'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS latest_timeline_details
FROM projects p
JOIN event_categories c ON c.id = p.category_id
LEFT JOIN LATERAL (
  SELECT t.id, t.at_utc
  FROM project_timeline t
  WHERE t.project_id = p.id
  ORDER BY t.at_utc DESC NULLS FIRST, t.sort_order ASC
  LIMIT 1
) lt ON TRUE
WHERE ($2::TEXT IS NULL OR $2 = '') OR (
  p.key ILIKE ('%' || $2 || '%')
  OR c.key ILIKE ('%' || $2 || '%')
  OR p.status_type ILIKE ('%' || $2 || '%')
  OR (
    p.end_utc < now()
    AND EXISTS (
      SELECT 1
      FROM unnest($1::TEXT[]) AS lp(locale)
      WHERE (lp.locale = 'fr' AND 'terminé' ILIKE ('%' || $2 || '%'))
         OR (lp.locale <> 'fr' AND 'finished' ILIKE ('%' || $2 || '%'))
    )
  )
  OR coalesce((
    SELECT ci.label
    FROM event_category_i18n ci
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ci.locale
    WHERE ci.category_id = c.id
    ORDER BY lp.ord
    LIMIT 1
  ), c.key) ILIKE ('%' || $2 || '%')
  OR coalesce((
    SELECT pi.field_value
    FROM project_i18n pi
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = pi.locale
    WHERE pi.project_id = p.id AND pi.field_key = 'title'
    ORDER BY lp.ord
    LIMIT 1
  ), '') ILIKE ('%' || $2 || '%')
  OR coalesce((
    SELECT pi.field_value
    FROM project_i18n pi
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = pi.locale
    WHERE pi.project_id = p.id AND pi.field_key = 'description'
    ORDER BY lp.ord
    LIMIT 1
  ), '') ILIKE ('%' || $2 || '%')
  OR coalesce((
    SELECT pi.field_value
    FROM project_i18n pi
    JOIN unnest($1::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = pi.locale
    WHERE pi.project_id = p.id AND pi.field_key = 'status_text'
    ORDER BY lp.ord
    LIMIT 1
  ), '') ILIKE ('%' || $2 || '%')
)
ORDER BY p.start_utc ASC NULLS FIRST, p.updated_at DESC
"#,
    )
    .bind(locale_chain)
    .bind(query)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(to_list_item).collect()
}

fn to_list_item(row: sqlx::postgres::PgRow) -> Result<ProjectListItem, AppError> {
    let start_utc: Option<DateTime<Utc>> = row.try_get("start_utc")?;
    let end_utc: Option<DateTime<Utc>> = row.try_get("end_utc")?;
    let latest_timeline_id: Option<Uuid> = row.try_get("latest_timeline_id")?;
    let latest_timeline_at_utc: Option<DateTime<Utc>> = row.try_get("latest_timeline_at_utc")?;
    let timeline = latest_timeline_id.map_or_else(Vec::new, |id| {
        vec![ProjectTimelineItem {
            id: id.to_string(),
            at_utc: latest_timeline_at_utc.map(|value| value.to_rfc3339()),
            title: row.try_get("latest_timeline_title").unwrap_or_default(),
            details: row.try_get("latest_timeline_details").unwrap_or_default(),
        }]
    });
    Ok(ProjectListItem {
        key: row.try_get("key")?,
        category_id: row.try_get("category_id")?,
        category: CategoryDisplay {
            id: row.try_get("category_id")?,
            key: row.try_get("category_display_key")?,
            icon: row.try_get("category_icon")?,
            color: row.try_get("category_color")?,
            label: row.try_get("category_label")?,
        },
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        start_utc: start_utc.map(|value| value.to_rfc3339()),
        end_utc: end_utc.map(|value| value.to_rfc3339()),
        status_type: row.try_get("status_type")?,
        status_text: row.try_get("status_text")?,
        timeline,
    })
}

pub async fn by_id(
    db: &sqlx::PgPool,
    project_code: &str,
    locale_chain: &[String],
) -> Result<Option<ProjectDetail>, AppError> {
    let row = sqlx::query(
        r#"
SELECT
  p.id,
  p.key,
  p.category_id::TEXT AS category_id,
  p.start_utc,
  p.end_utc,
  p.status_type,
  p.last_modified_at,
  u.id::TEXT AS last_modified_by_user_id,
  u.email AS last_modified_by_email,
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
  coalesce((
    SELECT pi.field_value
    FROM project_i18n pi
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = pi.locale
    WHERE pi.project_id = p.id AND pi.field_key = 'title'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS title,
  coalesce((
    SELECT pi.field_value
    FROM project_i18n pi
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = pi.locale
    WHERE pi.project_id = p.id AND pi.field_key = 'description'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS description
  , coalesce((
    SELECT pi.field_value
    FROM project_i18n pi
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = pi.locale
    WHERE pi.project_id = p.id AND pi.field_key = 'status_text'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS status_text
FROM projects p
JOIN event_categories c ON c.id = p.category_id
LEFT JOIN users u ON u.id = p.last_modified_by_user_id
WHERE p.key = $1
"#,
    )
    .bind(project_code)
    .bind(locale_chain)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else { return Ok(None); };

    let project_id: Uuid = row.try_get("id")?;
    let timeline_rows = sqlx::query(
        r#"
SELECT
  t.id,
  t.at_utc,
  coalesce((
    SELECT ti.field_value
    FROM project_timeline_i18n ti
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale
    WHERE ti.timeline_id = t.id AND ti.field_key = 'title'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS title,
  coalesce((
    SELECT ti.field_value
    FROM project_timeline_i18n ti
    JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale
    WHERE ti.timeline_id = t.id AND ti.field_key = 'details'
    ORDER BY lp.ord
    LIMIT 1
  ), '') AS details
FROM project_timeline t
WHERE t.project_id = $1
ORDER BY t.at_utc DESC NULLS FIRST, t.sort_order ASC
"#,
    )
    .bind(project_id)
    .bind(locale_chain)
    .fetch_all(db)
    .await?;

    let timeline = timeline_rows
        .into_iter()
        .map(|value| {
            let id: Uuid = value.try_get("id")?;
            let at_utc: Option<DateTime<Utc>> = value.try_get("at_utc")?;
            Ok(ProjectTimelineItem {
                id: id.to_string(),
                at_utc: at_utc.map(|value| value.to_rfc3339()),
                title: value.try_get("title")?,
                details: value.try_get("details")?,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    let start_utc: Option<DateTime<Utc>> = row.try_get("start_utc")?;
    let end_utc: Option<DateTime<Utc>> = row.try_get("end_utc")?;
    let last_modified_at: Option<DateTime<Utc>> = row.try_get("last_modified_at")?;
    let last_modified_by_user_id: Option<String> = row.try_get("last_modified_by_user_id")?;
    let last_modified_by_email: Option<String> = row.try_get("last_modified_by_email")?;

    Ok(Some(ProjectDetail {
        key: row.try_get("key")?,
        category_id: row.try_get("category_id")?,
        category: CategoryDisplay {
            id: row.try_get("category_id")?,
            key: row.try_get("category_display_key")?,
            icon: row.try_get("category_icon")?,
            color: row.try_get("category_color")?,
            label: row.try_get("category_label")?,
        },
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        start_utc: start_utc.map(|value| value.to_rfc3339()),
        end_utc: end_utc.map(|value| value.to_rfc3339()),
        status_type: row.try_get("status_type")?,
        status_text: row.try_get("status_text")?,
        timeline,
        last_modified_at: last_modified_at.map(|value| value.to_rfc3339()),
        last_modified_by: last_modified_by_user_id.zip(last_modified_by_email).map(|(id, email)| AuditUser {
            id,
            email,
        }),
    }))
}

const PROJECT_TIMELINE_FIELDS: [&str; 2] = ["title", "details"];

pub async fn edit_data(
    db: &sqlx::PgPool,
    project_code: &str,
    locale: &str,
    locale_chain: &[String],
    enabled_locales: Vec<String>,
) -> Result<Option<ProjectEditData>, AppError> {
    let row = sqlx::query("SELECT id, key, category_id::TEXT AS category_id, start_utc, end_utc, status_type FROM projects WHERE key = $1")
        .bind(project_code)
        .fetch_optional(db)
        .await?;
    let Some(row) = row else { return Ok(None); };
    let project_id: Uuid = row.try_get("id")?;
    let start_utc: Option<DateTime<Utc>> = row.try_get("start_utc")?;
    let end_utc: Option<DateTime<Utc>> = row.try_get("end_utc")?;
    let timeline_rows = sqlx::query(
        "SELECT id, at_utc, sort_order FROM project_timeline WHERE project_id = $1 ORDER BY at_utc DESC NULLS FIRST, sort_order ASC",
    )
    .bind(project_id)
    .fetch_all(db)
    .await?;
    let mut timeline = Vec::with_capacity(timeline_rows.len());
    for timeline_row in timeline_rows {
        let timeline_id: Uuid = timeline_row.try_get("id")?;
        let at_utc: Option<DateTime<Utc>> = timeline_row.try_get("at_utc")?;
        timeline.push(ProjectTimelineEditItem {
            id: timeline_id.to_string(),
            at_utc: at_utc.map(|value| value.to_rfc3339()),
            sort_order: timeline_row.try_get("sort_order")?,
            fields: crate::common::timeline::edit_timeline_fields(db, "project_timeline_i18n", timeline_id, locale, locale_chain, &PROJECT_TIMELINE_FIELDS).await?,
        });
    }
    Ok(Some(ProjectEditData {
        key: row.try_get("key")?,
        category_id: row.try_get("category_id")?,
        start_utc: start_utc.map(|value| value.to_rfc3339()),
        end_utc: end_utc.map(|value| value.to_rfc3339()),
        status_type: row.try_get("status_type")?,
        locale: locale.to_string(),
        enabled_locales,
        fields: edit_fields(db, &project_id, locale, locale_chain).await?,
        timeline,
    }))
}

async fn edit_fields(
    db: &sqlx::PgPool,
    project_id: &Uuid,
    locale: &str,
    locale_chain: &[String],
) -> Result<Vec<EditFieldValue>, AppError> {
    let mut result = Vec::new();
    for field_key in ["title", "description", "status_text"] {
        let exact: Option<String> = sqlx::query_scalar(
            "SELECT field_value FROM project_i18n WHERE project_id = $1 AND locale = $2 AND field_key = $3",
        )
        .bind(project_id)
        .bind(locale)
        .bind(field_key)
        .fetch_optional(db)
        .await?;
        let fallback = if exact.is_none() {
            sqlx::query(
                r#"
SELECT pi.locale, pi.field_value
FROM project_i18n pi
JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = pi.locale
WHERE pi.project_id = $1 AND pi.field_key = $3
ORDER BY lp.ord
LIMIT 1
"#,
            )
            .bind(project_id)
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
            field_key: field_key.to_string(),
            value: exact.clone().or_else(|| fallback_value.clone()).unwrap_or_default(),
            exact_value: exact,
            fallback_locale,
            fallback_value,
        });
    }
    Ok(result)
}

pub async fn save_partial(db: &sqlx::PgPool, payload: &ProjectSaveRequest, user_id: uuid::Uuid) -> Result<String, AppError> {
    let start_utc = payload
        .start_utc
        .as_deref()
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|_| AppError::bad_request("invalid project start_utc"))?
        .map(|value| value.with_timezone(&Utc));
    let end_utc = payload
        .end_utc
        .as_deref()
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|_| AppError::bad_request("invalid project end_utc"))?
        .map(|value| value.with_timezone(&Utc));

    let mut tx = db.begin().await?;
    let (project_id, key): (Uuid, String) = if let Some(key) = &payload.key {
        let row = sqlx::query(
            r#"
UPDATE projects
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
        .ok_or_else(|| AppError::not_found("project not found"))?;
        (row.try_get("id")?, row.try_get("key")?)
    } else {
        let row = sqlx::query(
            r#"
INSERT INTO projects (id, key, category_id, start_utc, end_utc, status_type, updated_at, last_modified_at, last_modified_by_user_id)
VALUES (gen_random_uuid(), 'PRJ-' || nextval('project_key_seq'), $1::uuid, $2, $3, $4, now(), now(), $5)
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
        sqlx::query(
            r#"
INSERT INTO project_i18n (project_id, locale, field_key, field_value)
VALUES ($1, $2, $3, $4)
ON CONFLICT (project_id, locale, field_key) DO UPDATE SET field_value = EXCLUDED.field_value
"#,
        )
        .bind(project_id)
        .bind(&payload.locale)
        .bind(field_key)
        .bind(field_value)
        .execute(&mut *tx)
        .await?;
    }

    let submitted_ids = payload
        .timeline
        .iter()
        .map(|item| Uuid::parse_str(&item.id).map_err(|_| AppError::bad_request("invalid timeline id")))
        .collect::<Result<Vec<_>, AppError>>()?;
    if !submitted_ids.is_empty() {
        let foreign_timeline_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM project_timeline WHERE id = ANY($1) AND project_id <> $2 LIMIT 1",
        )
        .bind(&submitted_ids)
        .bind(project_id)
        .fetch_optional(&mut *tx)
        .await?;
        if foreign_timeline_id.is_some() {
            return Err(AppError::bad_request("timeline id does not belong to project"));
        }
    }
    if payload.replace_timeline && !submitted_ids.is_empty() {
        sqlx::query("DELETE FROM project_timeline WHERE project_id = $1 AND NOT (id = ANY($2))")
            .bind(project_id)
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
INSERT INTO project_timeline (id, project_id, at_utc, sort_order)
VALUES ($1, $2, $3, $4)
ON CONFLICT (id) DO UPDATE SET at_utc = EXCLUDED.at_utc, sort_order = EXCLUDED.sort_order
WHERE project_timeline.project_id = EXCLUDED.project_id
"#,
        )
        .bind(timeline_id)
        .bind(project_id)
        .bind(at_utc)
        .bind(item.sort_order)
        .execute(&mut *tx)
        .await?;
        for (field_key, field_value) in &item.fields {
            let trimmed = field_value.trim();
            if trimmed.is_empty() {
                sqlx::query("DELETE FROM project_timeline_i18n WHERE timeline_id = $1 AND locale = $2 AND field_key = $3")
                    .bind(timeline_id)
                    .bind(&payload.locale)
                    .bind(field_key)
                    .execute(&mut *tx)
                    .await?;
            } else {
                sqlx::query(
                    r#"
INSERT INTO project_timeline_i18n (timeline_id, locale, field_key, field_value)
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

pub async fn list_translations(db: &sqlx::PgPool, project_code: &str) -> Result<Vec<ProjectTranslationMatrixRow>, AppError> {
    let project_exists: Option<bool> = sqlx::query_scalar("SELECT TRUE FROM projects WHERE key = $1")
        .bind(project_code)
        .fetch_optional(db)
        .await?;
    if project_exists.is_none() {
        return Err(AppError::not_found("project not found"));
    }

    let rows = sqlx::query(
        r#"
WITH target AS (
  SELECT id FROM projects WHERE key = $1
), keys AS (
  SELECT 'title' AS field_key
  UNION ALL SELECT 'description'
  UNION ALL SELECT 'status_text'
)
SELECT
  k.field_key,
  l.code AS locale,
  pi.field_value
FROM locales l
CROSS JOIN keys k
LEFT JOIN project_i18n pi
  ON pi.project_id = (SELECT id FROM target)
 AND pi.locale = l.code
 AND pi.field_key = k.field_key
ORDER BY k.field_key, l.code
"#,
    )
    .bind(project_code)
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(ProjectTranslationMatrixRow {
                field_key: row.try_get("field_key")?,
                locale: row.try_get("locale")?,
                field_value: row.try_get("field_value")?,
            })
        })
        .collect()
}

pub async fn replace_translations(
    db: &sqlx::PgPool,
    project_code: &str,
    values: &[ProjectTranslationValue],
) -> Result<(), AppError> {
    let project_id: String = sqlx::query_scalar("SELECT id::TEXT FROM projects WHERE key = $1")
        .bind(project_code)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::not_found("project not found"))?;

    let mut tx = db.begin().await?;
    sqlx::query("DELETE FROM project_i18n WHERE project_id = $1::uuid")
        .bind(&project_id)
        .execute(&mut *tx)
        .await?;

    for value in values {
        sqlx::query("INSERT INTO project_i18n (project_id, locale, field_key, field_value) VALUES ($1::uuid, $2, $3, $4)")
            .bind(&project_id)
            .bind(&value.locale)
            .bind(&value.field_key)
            .bind(&value.field_value)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn delete_by_code(db: &sqlx::PgPool, project_code: &str) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM projects WHERE key = $1")
        .bind(project_code)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
}
