use std::collections::HashMap;

use serde::Serialize;

#[derive(Serialize)]
pub struct CategoryDisplay {
    pub id: String,
    pub code: String,
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
pub struct ProjectListItem {
    pub id: String,
    pub category_code: String,
    pub category: CategoryDisplay,
    pub title: String,
    pub description: String,
    pub start_utc: Option<String>,
    pub end_utc: Option<String>,
    pub status_type: String,
    pub status_text: String,
}

#[derive(Serialize)]
pub struct ProjectDetail {
    pub id: String,
    pub category_code: String,
    pub category: CategoryDisplay,
    pub title: String,
    pub description: String,
    pub start_utc: Option<String>,
    pub end_utc: Option<String>,
    pub status_type: String,
    pub status_text: String,
    pub last_modified_at: Option<String>,
    pub last_modified_by: Option<AuditUser>,
}

#[derive(serde::Deserialize)]
pub struct ProjectListQuery {
    pub locale: Option<String>,
    pub q: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct ProjectTranslationsUpdateRequest {
    pub values: Vec<ProjectTranslationValue>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProjectTranslationValue {
    pub locale: String,
    pub field_key: String,
    pub field_value: String,
}

#[derive(serde::Serialize)]
pub struct ProjectTranslationMatrixRow {
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
pub struct ProjectEditData {
    pub id: String,
    pub category_id: String,
    pub start_utc: Option<String>,
    pub end_utc: Option<String>,
    pub status_type: String,
    pub locale: String,
    pub enabled_locales: Vec<String>,
    pub fields: Vec<EditFieldValue>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
pub struct ProjectSaveRequest {
    pub id: String,
    pub category_id: String,
    pub start_utc: Option<String>,
    pub end_utc: Option<String>,
    pub status_type: String,
    pub locale: String,
    pub fields: HashMap<String, String>,
}
