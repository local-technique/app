use std::collections::HashSet;

use chrono::{DateTime, Utc};

use crate::common::auth::Principal;
use crate::common::error::AppError;
use crate::common::i18n::locale_chain;
use crate::common::timeline;
use crate::common::validation::{
    ensure_field_key_allowed, ensure_locale_enabled, load_enabled_locales, normalize_field_key, normalize_locale,
    normalize_text_value, validate_field_map,
};
use crate::maintenances::model::{
    MaintenanceDetail, MaintenanceEditData, MaintenanceListItem, MaintenanceSaveRequest, MaintenanceTimelineCreateRequest,
    MaintenanceTimelineItem, MaintenanceTimelineSaveItem, MaintenanceTimelineUpdateRequest, MaintenanceTranslationMatrixRow,
    MaintenanceTranslationValue,
};
use crate::maintenances::repository;

const MAINTENANCE_STATUSES: [&str; 2] = ["waiting", "ongoing"];
const MAINTENANCE_TRANSLATION_FIELD_KEYS: [&str; 5] =
    ["title", "warning", "description", "location", "status_text"];
const MAINTENANCE_TIMELINE_TRANSLATION_FIELD_KEYS: [&str; 2] = ["title", "details"];

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
) -> Result<String, AppError> {
    let enabled_locales = load_enabled_locales(db).await?;
    let locale = normalize_locale(&payload.locale)?;
    ensure_locale_enabled(&locale, &enabled_locales)?;
    let status_type = normalize_text_value(&payload.status_type).to_lowercase();
    if !MAINTENANCE_STATUSES.contains(&status_type.as_str()) {
        return Err(AppError::bad_request("unsupported maintenance status"));
    }
    let mut fields = std::collections::HashMap::new();
    for (field_key, field_value) in &payload.fields {
        let field_key = normalize_field_key(field_key)?;
        ensure_field_key_allowed(&field_key, &MAINTENANCE_TRANSLATION_FIELD_KEYS)?;
        let value = normalize_text_value(field_value);
        if matches!(field_key.as_str(), "title" | "description") && value.is_empty() {
            return Err(AppError::bad_request("required localized fields cannot be empty"));
        }
        fields.insert(field_key, value);
    }
    for required in ["title", "description"] {
        if !fields.contains_key(required) {
            return Err(AppError::bad_request("required localized fields are missing"));
        }
    }
    let mut timeline = Vec::with_capacity(payload.timeline.len());
    for item in &payload.timeline {
        timeline.push(MaintenanceTimelineSaveItem {
            id: normalize_text_value(&item.id),
            at_utc: item.at_utc.clone(),
            sort_order: item.sort_order,
            fields: validate_field_map(&item.fields, &MAINTENANCE_TIMELINE_TRANSLATION_FIELD_KEYS, &["title"])?,
        });
    }

    let validated = MaintenanceSaveRequest {
        key: payload.key.as_deref().map(normalize_text_value).filter(|value| !value.is_empty()),
        category_id: normalize_text_value(&payload.category_id),
        start_utc: payload.start_utc.clone(),
        end_utc: payload.end_utc.clone(),
        status_type,
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

pub async fn delete(db: &sqlx::PgPool, maintenance_id: &str) -> Result<bool, AppError> {
    repository::delete_by_code(db, maintenance_id).await
}

pub async fn create_timeline_entry(
    db: &sqlx::PgPool,
    maintenance_code: &str,
    payload: &MaintenanceTimelineCreateRequest,
    locale: &str,
    user_id: uuid::Uuid,
) -> Result<MaintenanceTimelineItem, AppError> {
    let enabled_locales = load_enabled_locales(db).await?;
    let locale = normalize_locale(locale)?;
    ensure_locale_enabled(&locale, &enabled_locales)?;
    let fields = validate_field_map(&payload.fields, &MAINTENANCE_TIMELINE_TRANSLATION_FIELD_KEYS, &["title"])?;
    let at_utc = payload
        .at_utc
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|_| AppError::bad_request("invalid at_utc"))?
        .map(|v| v.with_timezone(&Utc));
    repository::create_timeline_entry(db, maintenance_code, at_utc, payload.sort_order, &locale, &fields, user_id).await
}

pub async fn update_timeline_entry(
    db: &sqlx::PgPool,
    maintenance_code: &str,
    entry_id: &str,
    payload: &MaintenanceTimelineUpdateRequest,
    locale: &str,
    principal: &Principal,
) -> Result<MaintenanceTimelineItem, AppError> {
    timeline::check_timeline_authorization(db, "maintenance_timeline", entry_id, principal).await?;
    let enabled_locales = load_enabled_locales(db).await?;
    let locale = normalize_locale(locale)?;
    ensure_locale_enabled(&locale, &enabled_locales)?;
    let fields = validate_field_map(&payload.fields, &MAINTENANCE_TIMELINE_TRANSLATION_FIELD_KEYS, &["title"])?;
    let at_utc = payload
        .at_utc
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|_| AppError::bad_request("invalid at_utc"))?
        .map(|v| v.with_timezone(&Utc));
    repository::update_timeline_entry(db, maintenance_code, entry_id, at_utc, payload.sort_order, &locale, &fields, principal.user_id)
        .await?
        .ok_or_else(|| AppError::not_found("timeline entry not found"))
}

pub async fn delete_timeline_entry(
    db: &sqlx::PgPool,
    maintenance_code: &str,
    entry_id: &str,
    principal: &Principal,
) -> Result<(), AppError> {
    timeline::check_timeline_authorization(db, "maintenance_timeline", entry_id, principal).await?;
    let deleted = repository::delete_timeline_entry(db, maintenance_code, entry_id).await?;
    if deleted {
        Ok(())
    } else {
        Err(AppError::not_found("timeline entry not found"))
    }
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
