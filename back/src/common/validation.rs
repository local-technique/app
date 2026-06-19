use std::collections::HashSet;

use serde::Serialize;
use utoipa::ToSchema;

use crate::common::error::AppError;

#[derive(Serialize, ToSchema)]
pub struct EditFieldValue {
    pub field_key: String,
    pub value: String,
    pub exact_value: Option<String>,
    pub fallback_locale: Option<String>,
    pub fallback_value: Option<String>,
}

pub fn validate_field_map(
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

pub fn normalize_locale(value: &str) -> Result<String, AppError> {
    let locale = value.trim().to_lowercase();
    if locale.is_empty() {
        return Err(AppError::bad_request("locale is required"));
    }
    if !is_valid_locale_code(&locale) {
        return Err(AppError::bad_request("invalid locale format"));
    }
    Ok(locale)
}

pub fn normalize_field_key(value: &str) -> Result<String, AppError> {
    let field_key = value.trim().to_lowercase();
    if field_key.is_empty() {
        return Err(AppError::bad_request("field_key is required"));
    }
    if !is_valid_identifier(&field_key) {
        return Err(AppError::bad_request("invalid field_key format"));
    }
    Ok(field_key)
}

pub fn normalize_key_name(value: &str) -> Result<String, AppError> {
    let key_name = value.trim().to_lowercase();
    if key_name.is_empty() {
        return Err(AppError::bad_request("key_name is required"));
    }
    if !is_valid_identifier(&key_name) {
        return Err(AppError::bad_request("invalid key_name format"));
    }
    Ok(key_name)
}

pub fn normalize_text_value(value: &str) -> String {
    value.trim().to_string()
}

pub fn ensure_field_key_allowed(field_key: &str, allowed_keys: &[&str]) -> Result<(), AppError> {
    if allowed_keys.contains(&field_key) {
        return Ok(());
    }
    Err(AppError::bad_request("unsupported field_key"))
}

pub async fn load_enabled_locales(db: &sqlx::PgPool) -> Result<HashSet<String>, AppError> {
    let rows = sqlx::query_scalar::<_, String>("SELECT code FROM locales WHERE is_enabled = TRUE")
        .fetch_all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|code| code.trim().to_lowercase())
        .collect())
}

pub fn ensure_locale_enabled(locale: &str, enabled_locales: &HashSet<String>) -> Result<(), AppError> {
    if enabled_locales.contains(locale) {
        return Ok(());
    }
    Err(AppError::bad_request("unsupported locale"))
}

fn is_valid_locale_code(value: &str) -> bool {
    if value.len() < 2 || value.len() > 15 {
        return false;
    }

    let mut last_was_separator = false;
    for c in value.chars() {
        if c == '-' || c == '_' {
            if last_was_separator {
                return false;
            }
            last_was_separator = true;
            continue;
        }

        if !c.is_ascii_alphanumeric() {
            return false;
        }
        last_was_separator = false;
    }

    !last_was_separator
}

fn is_valid_identifier(value: &str) -> bool {
    if value.len() > 128 {
        return false;
    }
    value.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}
