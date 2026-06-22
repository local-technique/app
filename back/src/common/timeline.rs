use sqlx::Row;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::common::validation::EditFieldValue;

pub async fn edit_timeline_fields(
    db: &sqlx::PgPool,
    table_name: &str,
    timeline_id: Uuid,
    locale: &str,
    locale_chain: &[String],
    field_keys: &[&str],
) -> Result<Vec<EditFieldValue>, AppError> {
    let exact_query = format!(
        "SELECT field_value FROM {} WHERE timeline_id = $1 AND locale = $2 AND field_key = $3",
        table_name
    );
    let fallback_query = format!(
        r#"
SELECT ti.locale, ti.field_value
FROM {} ti
JOIN unnest($2::TEXT[]) WITH ORDINALITY AS lp(locale, ord) ON lp.locale = ti.locale
WHERE ti.timeline_id = $1 AND ti.field_key = $3
ORDER BY lp.ord
LIMIT 1
"#,
        table_name
    );
    let mut result = Vec::with_capacity(field_keys.len());
    for field_key in field_keys {
        let exact: Option<String> = sqlx::query_scalar(&exact_query)
            .bind(timeline_id)
            .bind(locale)
            .bind(field_key)
            .fetch_optional(db)
            .await?;
        let fallback = if exact.is_none() {
            sqlx::query(&fallback_query)
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
