use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::common::error::AppError;
use crate::projects::model::{
    AuditUser, CategoryDisplay, EditFieldValue, ProjectDetail, ProjectEditData, ProjectListItem,
    ProjectSaveRequest, ProjectTranslationMatrixRow, ProjectTranslationValue,
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
  ), '') AS status_text
FROM projects p
JOIN event_categories c ON c.id = p.category_id
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
        last_modified_at: last_modified_at.map(|value| value.to_rfc3339()),
        last_modified_by: last_modified_by_user_id.zip(last_modified_by_email).map(|(id, email)| AuditUser {
            id,
            email,
        }),
    }))
}

pub async fn edit_data(
    db: &sqlx::PgPool,
    project_code: &str,
    locale: &str,
    locale_chain: &[String],
    enabled_locales: Vec<String>,
) -> Result<Option<ProjectEditData>, AppError> {
    let row = sqlx::query("SELECT id::TEXT AS id, key, category_id::TEXT AS category_id, start_utc, end_utc, status_type FROM projects WHERE key = $1")
        .bind(project_code)
        .fetch_optional(db)
        .await?;
    let Some(row) = row else { return Ok(None); };
    let project_id: String = row.try_get("id")?;
    let start_utc: Option<DateTime<Utc>> = row.try_get("start_utc")?;
    let end_utc: Option<DateTime<Utc>> = row.try_get("end_utc")?;

    Ok(Some(ProjectEditData {
        key: row.try_get("key")?,
        category_id: row.try_get("category_id")?,
        start_utc: start_utc.map(|value| value.to_rfc3339()),
        end_utc: end_utc.map(|value| value.to_rfc3339()),
        status_type: row.try_get("status_type")?,
        locale: locale.to_string(),
        enabled_locales,
        fields: edit_fields(db, &project_id, locale, locale_chain).await?,
    }))
}

async fn edit_fields(
    db: &sqlx::PgPool,
    project_id: &str,
    locale: &str,
    locale_chain: &[String],
) -> Result<Vec<EditFieldValue>, AppError> {
    let mut result = Vec::new();
    for field_key in ["title", "description", "status_text"] {
        let exact = sqlx::query_scalar::<_, String>(
            "SELECT field_value FROM project_i18n WHERE project_id = $1::uuid AND locale = $2 AND field_key = $3",
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
WHERE pi.project_id = $1::uuid AND pi.field_key = $3
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
    let (project_id, key): (String, String) = if let Some(key) = &payload.key {
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
RETURNING id::TEXT AS id, key
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
RETURNING id::TEXT AS id, key
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
VALUES ($1::uuid, $2, $3, $4)
ON CONFLICT (project_id, locale, field_key) DO UPDATE SET field_value = EXCLUDED.field_value
"#,
        )
        .bind(&project_id)
        .bind(&payload.locale)
        .bind(field_key)
        .bind(field_value)
        .execute(&mut *tx)
        .await?;
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
