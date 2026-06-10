use std::collections::HashSet;

use crate::common::error::AppError;
use crate::common::i18n::locale_chain;
use crate::common::validation::{
    ensure_field_key_allowed, ensure_locale_enabled, load_enabled_locales, normalize_field_key, normalize_locale,
    normalize_text_value,
};
use crate::incidents::model::{
    IncidentDetail, IncidentEditData, IncidentListItem, IncidentSaveRequest, IncidentTranslationMatrixRow,
    IncidentTranslationValue,
};
use crate::incidents::repository;

const INCIDENT_TRANSLATION_FIELD_KEYS: [&str; 4] = ["title", "short_description", "long_description", "location"];
const INCIDENT_TIMELINE_TRANSLATION_FIELD_KEYS: [&str; 2] = ["title", "details"];

pub async fn list(
    db: &sqlx::PgPool,
    requested_locale: Option<&str>,
    query: Option<&str>,
) -> Result<Vec<IncidentListItem>, AppError> {
    let chain = locale_chain(requested_locale);
    repository::list(db, &chain, query).await
}

pub async fn by_id(
    db: &sqlx::PgPool,
    incident_id: &str,
    requested_locale: Option<&str>,
) -> Result<Option<IncidentDetail>, AppError> {
    let chain = locale_chain(requested_locale);
    repository::by_id(db, incident_id, &chain).await
}

pub async fn list_translations(
    db: &sqlx::PgPool,
    incident_id: &str,
) -> Result<Vec<IncidentTranslationMatrixRow>, AppError> {
    repository::list_translations(db, incident_id).await
}

pub async fn edit_data(
    db: &sqlx::PgPool,
    incident_id: &str,
    requested_locale: Option<&str>,
) -> Result<Option<IncidentEditData>, AppError> {
    let locale = normalize_locale(requested_locale.unwrap_or("en"))?;
    let enabled_locales = load_enabled_locales(db).await?;
    ensure_locale_enabled(&locale, &enabled_locales)?;
    let mut enabled = enabled_locales.into_iter().collect::<Vec<_>>();
    enabled.sort();
    repository::edit_data(db, incident_id, &locale, &locale_chain(Some(&locale)), enabled).await
}

pub async fn replace_translations(
    db: &sqlx::PgPool,
    incident_id: &str,
    values: &[IncidentTranslationValue],
) -> Result<(), AppError> {
    let enabled_locales = load_enabled_locales(db).await?;
    let validated = values
        .iter()
        .map(|value| validate_translation_value(value, &enabled_locales, &INCIDENT_TRANSLATION_FIELD_KEYS))
        .collect::<Result<Vec<_>, AppError>>()?;

    repository::replace_translations(db, incident_id, &validated).await
}

pub async fn save_partial(
    db: &sqlx::PgPool,
    payload: &IncidentSaveRequest,
    user_id: uuid::Uuid,
) -> Result<String, AppError> {
    let enabled_locales = load_enabled_locales(db).await?;
    let locale = normalize_locale(&payload.locale)?;
    ensure_locale_enabled(&locale, &enabled_locales)?;
    let fields = validate_field_map(&payload.fields, &INCIDENT_TRANSLATION_FIELD_KEYS, &["title", "short_description", "long_description"])?;
    let mut timeline = Vec::with_capacity(payload.timeline.len());
    for item in &payload.timeline {
        timeline.push(crate::incidents::model::IncidentTimelineSaveItem {
            id: normalize_text_value(&item.id),
            at_utc: item.at_utc.clone(),
            sort_order: item.sort_order,
            fields: validate_field_map(&item.fields, &INCIDENT_TIMELINE_TRANSLATION_FIELD_KEYS, &["title"] )?,
        });
    }
    let validated = IncidentSaveRequest {
        key: payload.key.as_deref().map(normalize_text_value).filter(|value| !value.is_empty()),
        category_id: normalize_text_value(&payload.category_id),
        start_utc: payload.start_utc.clone(),
        end_utc: payload.end_utc.clone(),
        locale,
        fields,
        replace_timeline: payload.replace_timeline,
        timeline,
    };
    if validated.category_id.is_empty() {
        return Err(AppError::bad_request("category_id is required"));
    }
    repository::save_partial(db, &validated, user_id).await
}

pub async fn delete(db: &sqlx::PgPool, incident_id: &str) -> Result<bool, AppError> {
    repository::delete_by_code(db, incident_id).await
}

fn validate_translation_value(
    value: &IncidentTranslationValue,
    enabled_locales: &HashSet<String>,
    allowed_field_keys: &[&str],
) -> Result<IncidentTranslationValue, AppError> {
    let locale = normalize_locale(&value.locale)?;
    ensure_locale_enabled(&locale, enabled_locales)?;

    let field_key = normalize_field_key(&value.field_key)?;
    ensure_field_key_allowed(&field_key, allowed_field_keys)?;

    Ok(IncidentTranslationValue {
        locale,
        field_key,
        field_value: normalize_text_value(&value.field_value),
    })
}

fn validate_field_map(
    input: &std::collections::HashMap<String, String>,
    allowed_field_keys: &[&str],
    required_field_keys: &[&str],
) -> Result<std::collections::HashMap<String, String>, AppError> {
    let mut result = std::collections::HashMap::new();
    for (field_key, field_value) in input {
        let field_key = normalize_field_key(field_key)?;
        ensure_field_key_allowed(&field_key, allowed_field_keys)?;
        let value = normalize_text_value(field_value);
        if required_field_keys.contains(&field_key.as_str()) && value.is_empty() {
            return Err(AppError::bad_request("required localized fields cannot be empty"));
        }
        result.insert(field_key, value);
    }
    for required in required_field_keys {
        if !result.contains_key(*required) {
            return Err(AppError::bad_request("required localized fields are missing"));
        }
    }
    Ok(result)
}
