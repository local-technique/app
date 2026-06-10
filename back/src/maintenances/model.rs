use std::collections::HashMap;

use serde::Serialize;

#[derive(Serialize)]
pub struct CategoryDisplay {
    pub id: String,
    pub key: String,
    pub icon: String,
    pub color: String,
    pub label: String,
}

#[derive(Serialize)]
pub struct AuditUser {
    pub id: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct MaintenanceListItem {
    pub key: String,
    pub category_id: String,
    pub category: CategoryDisplay,
    pub title: String,
    pub warning: String,
    pub short_description: String,
    pub location: String,
    pub start_utc: String,
    pub end_utc: Option<String>,
    pub notified_at_utc: Option<String>,
}

#[derive(Serialize)]
pub struct MaintenanceDetail {
    pub key: String,
    pub category_id: String,
    pub category: CategoryDisplay,
    pub title: String,
    pub warning: String,
    pub short_description: String,
    pub long_description: String,
    pub location: String,
    pub start_utc: String,
    pub end_utc: Option<String>,
    pub notified_at_utc: Option<String>,
    pub last_modified_at: Option<String>,
    pub last_modified_by: Option<AuditUser>,
}

#[derive(serde::Deserialize)]
pub struct MaintenanceListQuery {
    pub locale: Option<String>,
    pub q: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct MaintenanceTranslationsUpdateRequest {
    pub values: Vec<MaintenanceTranslationValue>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct MaintenanceTranslationValue {
    pub locale: String,
    pub field_key: String,
    pub field_value: String,
}

#[derive(serde::Serialize)]
pub struct MaintenanceTranslationMatrixRow {
    pub field_key: String,
    pub locale: String,
    pub field_value: Option<String>,
}

#[derive(serde::Serialize)]
pub struct EditFieldValue {
    pub field_key: String,
    pub value: String,
    pub exact_value: Option<String>,
    pub fallback_locale: Option<String>,
    pub fallback_value: Option<String>,
}

#[derive(serde::Serialize)]
pub struct MaintenanceEditData {
    pub key: String,
    pub category_id: String,
    pub start_utc: String,
    pub end_utc: Option<String>,
    pub notified_at_utc: Option<String>,
    pub locale: String,
    pub enabled_locales: Vec<String>,
    pub fields: Vec<EditFieldValue>,
}

#[derive(serde::Deserialize)]
pub struct MaintenanceSaveRequest {
    #[serde(default)]
    pub key: Option<String>,
    pub category_id: String,
    pub start_utc: String,
    pub end_utc: Option<String>,
    pub notified_at_utc: Option<String>,
    pub locale: String,
    pub fields: HashMap<String, String>,
}

#[derive(serde::Serialize)]
pub struct CreatedKeyResponse {
    pub key: String,
}
