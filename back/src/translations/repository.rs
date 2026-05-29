use sqlx::Row;

use crate::common::error::AppError;
use crate::translations::model::{BulkTranslationValue, TranslationMatrixRow};

pub async fn list_matrix(db: &sqlx::PgPool) -> Result<Vec<TranslationMatrixRow>, AppError> {
    let rows = sqlx::query(
        r#"
SELECT
  k.key_name,
  l.code AS locale,
  v.value,
  (v.value IS NULL) AS is_missing
FROM translation_keys k
CROSS JOIN locales l
LEFT JOIN translation_values v
  ON v.key_name = k.key_name
 AND v.locale = l.code
ORDER BY k.key_name, l.code
"#,
    )
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(TranslationMatrixRow {
                key_name: row.try_get("key_name")?,
                locale: row.try_get("locale")?,
                value: row.try_get("value")?,
                is_missing: row.try_get("is_missing")?,
            })
        })
        .collect()
}

pub async fn upsert_bulk(db: &sqlx::PgPool, values: &[BulkTranslationValue]) -> Result<(), AppError> {
    let mut tx = db.begin().await?;

    for value in values {
        let key_name = value.key_name.trim();
        let locale = value.locale.trim().to_lowercase();
        let val = value.value.trim();

        sqlx::query(
            "INSERT INTO translation_keys (key_name) VALUES ($1) ON CONFLICT (key_name) DO NOTHING",
        )
        .bind(key_name)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
INSERT INTO translation_values (key_name, locale, value, updated_at)
VALUES ($1, $2, $3, now())
ON CONFLICT (key_name, locale)
DO UPDATE SET value = EXCLUDED.value, updated_at = now()
"#,
        )
        .bind(key_name)
        .bind(locale)
        .bind(val)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
