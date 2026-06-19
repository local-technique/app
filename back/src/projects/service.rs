use std::collections::HashSet;

use chrono::{DateTime, Utc};

use crate::common::error::AppError;
use crate::common::i18n::locale_chain;
use crate::common::validation::{
    ensure_field_key_allowed, ensure_locale_enabled, load_enabled_locales, normalize_field_key, normalize_locale,
    normalize_text_value, validate_field_map,
};
use crate::projects::model::{
    ProjectDetail, ProjectEditData, ProjectListItem, ProjectSaveRequest,
    ProjectTranslationMatrixRow, ProjectTranslationValue,
};
use crate::projects::repository;

const PROJECT_TRANSLATION_FIELD_KEYS: [&str; 3] = ["title", "description", "status_text"];
const PROJECT_TIMELINE_TRANSLATION_FIELD_KEYS: [&str; 2] = ["title", "details"];
const PROJECT_STATUSES: [&str; 2] = ["waiting", "ongoing"];

pub async fn list(
    db: &sqlx::PgPool,
    requested_locale: Option<&str>,
    query: Option<&str>,
) -> Result<Vec<ProjectListItem>, AppError> {
    let chain = locale_chain(requested_locale);
    repository::list(db, &chain, query).await
}

pub async fn by_id(
    db: &sqlx::PgPool,
    project_id: &str,
    requested_locale: Option<&str>,
) -> Result<Option<ProjectDetail>, AppError> {
    let chain = locale_chain(requested_locale);
    repository::by_id(db, project_id, &chain).await
}

pub async fn list_translations(
    db: &sqlx::PgPool,
    project_id: &str,
) -> Result<Vec<ProjectTranslationMatrixRow>, AppError> {
    repository::list_translations(db, project_id).await
}

pub async fn edit_data(
    db: &sqlx::PgPool,
    project_id: &str,
    requested_locale: Option<&str>,
) -> Result<Option<ProjectEditData>, AppError> {
    let locale = normalize_locale(requested_locale.unwrap_or("en"))?;
    let enabled_locales = load_enabled_locales(db).await?;
    ensure_locale_enabled(&locale, &enabled_locales)?;
    let mut enabled = enabled_locales.into_iter().collect::<Vec<_>>();
    enabled.sort();
    repository::edit_data(db, project_id, &locale, &locale_chain(Some(&locale)), enabled).await
}

pub async fn replace_translations(
    db: &sqlx::PgPool,
    project_id: &str,
    values: &[ProjectTranslationValue],
) -> Result<(), AppError> {
    let enabled_locales = load_enabled_locales(db).await?;
    let validated = values
        .iter()
        .map(|value| validate_translation_value(value, &enabled_locales))
        .collect::<Result<Vec<_>, AppError>>()?;

    repository::replace_translations(db, project_id, &validated).await
}

pub async fn save_partial(
    db: &sqlx::PgPool,
    payload: &ProjectSaveRequest,
    user_id: uuid::Uuid,
) -> Result<String, AppError> {
    let enabled_locales = load_enabled_locales(db).await?;
    let validated = validate_save_payload(payload, &enabled_locales)?;
    repository::save_partial(db, &validated, user_id).await
}

pub async fn delete(db: &sqlx::PgPool, project_id: &str) -> Result<bool, AppError> {
    repository::delete_by_code(db, project_id).await
}

fn validate_save_payload(
    payload: &ProjectSaveRequest,
    enabled_locales: &HashSet<String>,
) -> Result<ProjectSaveRequest, AppError> {
    let locale = normalize_locale(&payload.locale)?;
    ensure_locale_enabled(&locale, enabled_locales)?;

    let status_type = normalize_text_value(&payload.status_type).to_lowercase();
    if !PROJECT_STATUSES.contains(&status_type.as_str()) {
        return Err(AppError::bad_request("unsupported project status"));
    }

    let start_utc = validate_optional_datetime(payload.start_utc.as_deref(), "invalid project start_utc")?;
    let end_utc = validate_optional_datetime(payload.end_utc.as_deref(), "invalid project end_utc")?;
    if let (Some(start), Some(end)) = (start_utc, end_utc)
        && end < start
    {
        return Err(AppError::bad_request("project end_utc cannot be earlier than start_utc"));
    }

    let mut fields = std::collections::HashMap::new();
    for (field_key, field_value) in &payload.fields {
        let field_key = normalize_field_key(field_key)?;
        ensure_field_key_allowed(&field_key, &PROJECT_TRANSLATION_FIELD_KEYS)?;
        let value = normalize_text_value(field_value);
        if matches!(field_key.as_str(), "title" | "description" | "status_text") && value.is_empty() {
            return Err(AppError::bad_request("required localized fields cannot be empty"));
        }
        fields.insert(field_key, value);
    }
    for required in PROJECT_TRANSLATION_FIELD_KEYS {
        if !fields.contains_key(required) {
            return Err(AppError::bad_request("required localized fields are missing"));
        }
    }

    let mut timeline = Vec::with_capacity(payload.timeline.len());
    for item in &payload.timeline {
        timeline.push(crate::projects::model::ProjectTimelineSaveItem {
            id: normalize_text_value(&item.id),
            at_utc: item.at_utc.clone(),
            sort_order: item.sort_order,
            fields: validate_field_map(&item.fields, &PROJECT_TIMELINE_TRANSLATION_FIELD_KEYS, &["title"])?,
        });
    }

    let validated = ProjectSaveRequest {
        key: payload.key.as_deref().map(normalize_text_value).filter(|value| !value.is_empty()),
        category_id: normalize_text_value(&payload.category_id),
        start_utc: start_utc.map(|value| value.to_rfc3339()),
        end_utc: end_utc.map(|value| value.to_rfc3339()),
        status_type,
        locale,
        fields,
        replace_timeline: payload.replace_timeline,
        timeline,
    };
    if validated.category_id.is_empty() {
        return Err(AppError::bad_request("category_id is required"));
    }
    Ok(validated)
}

fn validate_optional_datetime(value: Option<&str>, error_message: &str) -> Result<Option<DateTime<Utc>>, AppError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|_| AppError::bad_request(error_message))
        .map(|value| value.map(|value| value.with_timezone(&Utc)))
}

fn validate_translation_value(
    value: &ProjectTranslationValue,
    enabled_locales: &HashSet<String>,
) -> Result<ProjectTranslationValue, AppError> {
    let locale = normalize_locale(&value.locale)?;
    ensure_locale_enabled(&locale, enabled_locales)?;

    let field_key = normalize_field_key(&value.field_key)?;
    ensure_field_key_allowed(&field_key, &PROJECT_TRANSLATION_FIELD_KEYS)?;

    Ok(ProjectTranslationValue {
        locale,
        field_key,
        field_value: normalize_text_value(&value.field_value),
    })
}

#[cfg(test)]
fn validate_save_payload_for_test(
    payload: &ProjectSaveRequest,
    enabled_locales: &HashSet<String>,
) -> Result<ProjectSaveRequest, AppError> {
    validate_save_payload(payload, enabled_locales)
}

#[cfg(test)]
fn validate_translation_value_for_test(
    value: &ProjectTranslationValue,
    enabled_locales: &HashSet<String>,
) -> Result<ProjectTranslationValue, AppError> {
    validate_translation_value(value, enabled_locales)
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::projects::model::{ProjectSaveRequest, ProjectTranslationValue};

    fn valid_payload() -> ProjectSaveRequest {
        ProjectSaveRequest {
            key: Some(" PRJ-001 ".to_string()),
            category_id: "ELV".to_string(),
            start_utc: Some("2026-01-10T10:00:00Z".to_string()),
            end_utc: Some("2026-02-10T10:00:00Z".to_string()),
            status_type: "ongoing".to_string(),
            locale: "en".to_string(),
            fields: HashMap::from([
                ("title".to_string(), "Elevator modernization".to_string()),
                ("description".to_string(), "Replace controller".to_string()),
                ("status_text".to_string(), "Installing controller".to_string()),
            ]),
            replace_timeline: false,
            timeline: vec![],
        }
    }

    fn enabled_locales() -> HashSet<String> {
        HashSet::from(["en".to_string(), "fr".to_string()])
    }

    #[test]
    fn validate_save_payload_trims_values_and_accepts_valid_status() {
        let validated = super::validate_save_payload_for_test(&valid_payload(), &enabled_locales()).expect("valid payload");

        assert_eq!(validated.key.as_deref(), Some("PRJ-001"));
        assert_eq!(validated.status_type, "ongoing");
        assert_eq!(validated.fields["title"], "Elevator modernization");
        assert_eq!(validated.fields["status_text"], "Installing controller");
    }

    #[test]
    fn validate_save_payload_rejects_missing_required_fields() {
        let mut payload = valid_payload();
        payload.fields.insert("description".to_string(), " ".to_string());

        let error = super::validate_save_payload_for_test(&payload, &enabled_locales()).expect_err("invalid payload");

        assert_eq!(error.message, "required localized fields cannot be empty");
    }

    #[test]
    fn validate_save_payload_rejects_missing_status_text() {
        let mut payload = valid_payload();
        payload.fields.remove("status_text");

        let error = super::validate_save_payload_for_test(&payload, &enabled_locales()).expect_err("missing status text");

        assert_eq!(error.message, "required localized fields are missing");
    }

    #[test]
    fn validate_save_payload_rejects_invalid_status() {
        let mut payload = valid_payload();
        payload.status_type = "finished".to_string();

        let error = super::validate_save_payload_for_test(&payload, &enabled_locales()).expect_err("invalid status");

        assert_eq!(error.message, "unsupported project status");
    }

    #[test]
    fn validate_save_payload_rejects_end_before_start() {
        let mut payload = valid_payload();
        payload.end_utc = Some("2026-01-01T10:00:00Z".to_string());

        let error = super::validate_save_payload_for_test(&payload, &enabled_locales()).expect_err("invalid dates");

        assert_eq!(error.message, "project end_utc cannot be earlier than start_utc");
    }

    #[test]
    fn validate_translation_value_rejects_unsupported_field_key() {
        let value = ProjectTranslationValue {
            locale: "en".to_string(),
            field_key: "summary".to_string(),
            field_value: "Text".to_string(),
        };

        let error = super::validate_translation_value_for_test(&value, &enabled_locales()).expect_err("invalid field");

        assert_eq!(error.message, "unsupported field_key");
    }
}
