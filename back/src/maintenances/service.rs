use std::collections::HashSet;

use crate::common::error::AppError;
use crate::common::i18n::locale_chain;
use crate::common::validation::{
    ensure_field_key_allowed, ensure_locale_enabled, load_enabled_locales, normalize_field_key, normalize_locale,
    normalize_text_value,
};
use crate::maintenances::model::{
    MaintenanceDetail, MaintenanceEditData, MaintenanceListItem, MaintenanceSaveRequest, MaintenanceTranslationMatrixRow,
    MaintenanceTranslationValue,
};
use crate::maintenances::repository;

const MAINTENANCE_TRANSLATION_FIELD_KEYS: [&str; 5] =
    ["title", "warning", "short_description", "long_description", "location"];

pub async fn list(
    db: &sqlx::PgPool,
    requested_locale: Option<&str>,
    query: Option<&str>,
) -> Result<Vec<MaintenanceListItem>, AppError> {
    let chain = locale_chain(requested_locale);
    repository::list(db, &chain, query).await
}

pub async fn by_id(
    db: &sqlx::PgPool,
    maintenance_id: &str,
    requested_locale: Option<&str>,
) -> Result<Option<MaintenanceDetail>, AppError> {
    let chain = locale_chain(requested_locale);
    repository::by_id(db, maintenance_id, &chain).await
}

pub async fn list_translations(
    db: &sqlx::PgPool,
    maintenance_id: &str,
) -> Result<Vec<MaintenanceTranslationMatrixRow>, AppError> {
    repository::list_translations(db, maintenance_id).await
}

pub async fn edit_data(
    db: &sqlx::PgPool,
    maintenance_id: &str,
    requested_locale: Option<&str>,
) -> Result<Option<MaintenanceEditData>, AppError> {
    let locale = normalize_locale(requested_locale.unwrap_or("en"))?;
    let enabled_locales = load_enabled_locales(db).await?;
    ensure_locale_enabled(&locale, &enabled_locales)?;
    let mut enabled = enabled_locales.into_iter().collect::<Vec<_>>();
    enabled.sort();
    repository::edit_data(db, maintenance_id, &locale, &locale_chain(Some(&locale)), enabled).await
}

pub async fn replace_translations(
    db: &sqlx::PgPool,
    maintenance_id: &str,
    values: &[MaintenanceTranslationValue],
) -> Result<(), AppError> {
    let enabled_locales = load_enabled_locales(db).await?;
    let validated = values
        .iter()
        .map(|value| validate_translation_value(value, &enabled_locales))
        .collect::<Result<Vec<_>, AppError>>()?;

    repository::replace_translations(db, maintenance_id, &validated).await
}

pub async fn save_partial(
    db: &sqlx::PgPool,
    payload: &MaintenanceSaveRequest,
    user_id: uuid::Uuid,
) -> Result<(), AppError> {
    let enabled_locales = load_enabled_locales(db).await?;
    let locale = normalize_locale(&payload.locale)?;
    ensure_locale_enabled(&locale, &enabled_locales)?;
    let mut fields = std::collections::HashMap::new();
    for (field_key, field_value) in &payload.fields {
        let field_key = normalize_field_key(field_key)?;
        ensure_field_key_allowed(&field_key, &MAINTENANCE_TRANSLATION_FIELD_KEYS)?;
        let value = normalize_text_value(field_value);
        if matches!(field_key.as_str(), "title" | "short_description" | "long_description") && value.is_empty() {
            return Err(AppError::bad_request("required localized fields cannot be empty"));
        }
        fields.insert(field_key, value);
    }
    for required in ["title", "short_description", "long_description"] {
        if !fields.contains_key(required) {
            return Err(AppError::bad_request("required localized fields are missing"));
        }
    }
    let validated = MaintenanceSaveRequest {
        id: normalize_text_value(&payload.id),
        category_id: normalize_text_value(&payload.category_id),
        start_utc: payload.start_utc.clone(),
        end_utc: payload.end_utc.clone(),
        notified_at_utc: payload.notified_at_utc.clone(),
        locale,
        fields,
    };
    if validated.id.is_empty() || validated.category_id.is_empty() {
        return Err(AppError::bad_request("maintenance id and category_id are required"));
    }
    repository::save_partial(db, &validated, user_id).await
}

pub async fn delete(db: &sqlx::PgPool, maintenance_id: &str) -> Result<bool, AppError> {
    repository::delete_by_code(db, maintenance_id).await
}

fn validate_translation_value(
    value: &MaintenanceTranslationValue,
    enabled_locales: &HashSet<String>,
) -> Result<MaintenanceTranslationValue, AppError> {
    let locale = normalize_locale(&value.locale)?;
    ensure_locale_enabled(&locale, enabled_locales)?;

    let field_key = normalize_field_key(&value.field_key)?;
    ensure_field_key_allowed(&field_key, &MAINTENANCE_TRANSLATION_FIELD_KEYS)?;

    Ok(MaintenanceTranslationValue {
        locale,
        field_key,
        field_value: normalize_text_value(&value.field_value),
    })
}
