use std::collections::HashSet;

use crate::common::error::AppError;
use crate::common::validation::{
    ensure_locale_enabled, load_enabled_locales, normalize_key_name, normalize_locale, normalize_text_value,
};
use crate::translations::model::{BulkTranslationValue, TranslationMatrixRow};
use crate::translations::repository;

pub async fn list_matrix(db: &sqlx::PgPool) -> Result<Vec<TranslationMatrixRow>, AppError> {
    repository::list_matrix(db).await
}

pub async fn upsert_bulk(db: &sqlx::PgPool, values: &[BulkTranslationValue]) -> Result<(), AppError> {
    let enabled_locales = load_enabled_locales(db).await?;
    let validated = values
        .iter()
        .map(|value| validate_bulk_value(value, &enabled_locales))
        .collect::<Result<Vec<_>, AppError>>()?;

    repository::upsert_bulk(db, &validated).await
}

fn validate_bulk_value(
    value: &BulkTranslationValue,
    enabled_locales: &HashSet<String>,
) -> Result<BulkTranslationValue, AppError> {
    let locale = normalize_locale(&value.locale)?;
    ensure_locale_enabled(&locale, enabled_locales)?;

    Ok(BulkTranslationValue {
        key_name: normalize_key_name(&value.key_name)?,
        locale,
        value: normalize_text_value(&value.value),
    })
}
