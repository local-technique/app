use std::collections::{HashMap, HashSet};

use crate::categories::model::{CategoryCreateRequest, CategoryItem, CategoryUpdateRequest};
use crate::categories::repository;
use crate::common::error::AppError;
use crate::common::i18n::locale_chain;
use crate::common::validation::{load_enabled_locales, normalize_locale, normalize_text_value};

pub async fn list(db: &sqlx::PgPool, requested_locale: Option<&str>) -> Result<Vec<CategoryItem>, AppError> {
    repository::list(db, &locale_chain(requested_locale)).await
}

pub async fn create(db: &sqlx::PgPool, payload: &CategoryCreateRequest) -> Result<(), AppError> {
    let enabled = load_enabled_locales(db).await?;
    let id = normalize_required(&payload.id, "category id is required")?;
    let code = normalize_required(&payload.code, "category code is required")?;
    let icon = normalize_required(&payload.icon, "category icon is required")?;
    let labels = validate_labels(&payload.labels, &enabled)?;
    repository::create(db, &id, &code, &icon, &labels).await
}

pub async fn update(db: &sqlx::PgPool, id: &str, payload: &CategoryUpdateRequest) -> Result<(), AppError> {
    let enabled = load_enabled_locales(db).await?;
    let id = normalize_required(id, "category id is required")?;
    let code = normalize_required(&payload.code, "category code is required")?;
    let icon = normalize_required(&payload.icon, "category icon is required")?;
    let labels = validate_labels(&payload.labels, &enabled)?;
    repository::update(db, &id, &code, &icon, &labels).await
}

pub async fn delete(db: &sqlx::PgPool, id: &str) -> Result<(), AppError> {
    repository::delete(db, &normalize_required(id, "category id is required")?).await
}

fn normalize_required(value: &str, message: &str) -> Result<String, AppError> {
    let value = normalize_text_value(value);
    if value.is_empty() {
        Err(AppError::bad_request(message))
    } else {
        Ok(value)
    }
}

fn validate_labels(
    labels: &HashMap<String, String>,
    enabled: &HashSet<String>,
) -> Result<HashMap<String, String>, AppError> {
    let mut result = HashMap::new();
    for locale in enabled {
        let value = labels
            .get(locale)
            .or_else(|| labels.get(&normalize_locale(locale).ok()?))
            .map(|value| normalize_text_value(value))
            .unwrap_or_default();
        if value.is_empty() {
            return Err(AppError::bad_request("category labels are required for all enabled locales"));
        }
        result.insert(locale.clone(), value);
    }
    Ok(result)
}
