use std::collections::{HashMap, HashSet};

use crate::categories::model::{CategoryCreateRequest, CategoryItem, CategoryUpdateRequest};
use crate::categories::repository;
use crate::common::error::AppError;
use crate::common::i18n::locale_chain;
use crate::common::validation::{load_enabled_locales, normalize_locale, normalize_text_value};

pub async fn list(db: &sqlx::PgPool, requested_locale: Option<&str>) -> Result<Vec<CategoryItem>, AppError> {
    repository::list(db, &locale_chain(requested_locale)).await
}

pub async fn create(db: &sqlx::PgPool, payload: &CategoryCreateRequest) -> Result<CategoryItem, AppError> {
    let enabled = load_enabled_locales(db).await?;
    let key = normalize_category_key(&payload.key)?;
    let icon = normalize_required(&payload.icon, "category icon is required")?;
    let color = normalize_color(&payload.color)?;
    let labels = validate_labels(&payload.labels, &enabled)?;
    repository::create(db, &key, &icon, &color, &labels, &locale_chain(None)).await
}

pub async fn update(db: &sqlx::PgPool, id: &str, payload: &CategoryUpdateRequest) -> Result<CategoryItem, AppError> {
    let enabled = load_enabled_locales(db).await?;
    let key = normalize_category_key(&payload.key)?;
    let icon = normalize_required(&payload.icon, "category icon is required")?;
    let color = normalize_color(&payload.color)?;
    let labels = validate_labels(&payload.labels, &enabled)?;
    repository::update(db, id, &key, &icon, &color, &labels, &locale_chain(None)).await
}

pub async fn delete(db: &sqlx::PgPool, id: &str) -> Result<(), AppError> {
    repository::delete(db, id).await
}

fn normalize_required(value: &str, message: &str) -> Result<String, AppError> {
    let value = normalize_text_value(value);
    if value.is_empty() {
        Err(AppError::bad_request(message))
    } else {
        Ok(value)
    }
}

fn normalize_color(value: &str) -> Result<String, AppError> {
    let value = normalize_text_value(value).to_lowercase();
    let valid = value.len() == 7
        && value.starts_with('#')
        && value.chars().skip(1).all(|ch| ch.is_ascii_hexdigit());
    if valid {
        Ok(value)
    } else {
        Err(AppError::bad_request("category color must be a #RRGGBB hex value"))
    }
}

fn normalize_category_key(value: &str) -> Result<String, AppError> {
    let value = normalize_text_value(value).to_uppercase();
    let valid = (3..=5).contains(&value.len()) && value.chars().all(|ch| ch.is_ascii_alphanumeric());
    if valid {
        Ok(value)
    } else {
        Err(AppError::bad_request("category key must be 3 to 5 ASCII alphanumeric characters"))
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

#[cfg(test)]
fn normalize_color_for_test(value: &str) -> Result<String, AppError> {
    normalize_color(value)
}

#[cfg(test)]
fn normalize_category_key_for_test(value: &str) -> Result<String, AppError> {
    normalize_category_key(value)
}

#[cfg(test)]
mod tests {
    #[test]
    fn normalize_color_accepts_hex_and_lowercases_it() {
        assert_eq!(super::normalize_color_for_test(" #9AAAB1 ").expect("valid color"), "#9aaab1");
    }

    #[test]
    fn normalize_color_rejects_invalid_values() {
        let error = super::normalize_color_for_test("blue").expect_err("invalid color");
        assert_eq!(error.message, "category color must be a #RRGGBB hex value");
    }

    #[test]
    fn normalize_category_key_uppercases_trimmed_ascii_key() {
        assert_eq!(super::normalize_category_key_for_test(" hea ").expect("valid key"), "HEA");
    }

    #[test]
    fn normalize_category_key_rejects_invalid_values() {
        for value in ["AB", "ABCDEF", "A-B", "ééé"] {
            let error = super::normalize_category_key_for_test(value).expect_err("invalid key");
            assert_eq!(error.message, "category key must be 3 to 5 ASCII alphanumeric characters");
        }
    }
}
