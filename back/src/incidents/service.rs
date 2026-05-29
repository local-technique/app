use std::collections::HashSet;

use crate::common::error::AppError;
use crate::common::i18n::locale_chain;
use crate::common::validation::{
    ensure_field_key_allowed, ensure_locale_enabled, load_enabled_locales, normalize_field_key, normalize_locale,
    normalize_text_value,
};
use crate::incidents::model::{
    IncidentDetail, IncidentListItem, IncidentTranslationMatrixRow, IncidentTranslationValue,
    IncidentUpsertRequest,
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

pub async fn upsert(db: &sqlx::PgPool, payload: &IncidentUpsertRequest) -> Result<(), AppError> {
    let enabled_locales = load_enabled_locales(db).await?;
    let mut validated = IncidentUpsertRequest {
        id: payload.id.clone(),
        category_code: payload.category_code.clone(),
        start_utc: payload.start_utc.clone(),
        end_utc: payload.end_utc.clone(),
        translations: payload
            .translations
            .iter()
            .map(|value| validate_translation_value(value, &enabled_locales, &INCIDENT_TRANSLATION_FIELD_KEYS))
            .collect::<Result<Vec<_>, AppError>>()?,
        timeline: Vec::with_capacity(payload.timeline.len()),
    };

    for item in &payload.timeline {
        validated.timeline.push(crate::incidents::model::IncidentTimelineUpsertItem {
            id: item.id.clone(),
            at_utc: item.at_utc.clone(),
            sort_order: item.sort_order,
            translations: item
                .translations
                .iter()
                .map(|value| {
                    validate_translation_value(value, &enabled_locales, &INCIDENT_TIMELINE_TRANSLATION_FIELD_KEYS)
                })
                .collect::<Result<Vec<_>, AppError>>()?,
        });
    }

    repository::upsert(db, &validated).await
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
