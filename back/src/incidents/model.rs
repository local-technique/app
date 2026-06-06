use std::collections::HashMap;

use serde::Serialize;

#[derive(Serialize)]
pub struct CategoryDisplay {
    pub id: String,
    pub code: String,
    pub icon: String,
    pub label: String,
}

#[derive(Serialize)]
pub struct AuditUser {
    pub id: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct IncidentListItem {
    pub id: String,
    pub category_code: String,
    pub category: CategoryDisplay,
    pub title: String,
    pub short_description: String,
    pub location: String,
    pub start_utc: String,
    pub end_utc: Option<String>,
    pub timeline: Vec<IncidentTimelineItem>,
}

#[derive(Serialize)]
pub struct IncidentTimelineItem {
    pub id: String,
    pub at_utc: Option<String>,
    pub title: String,
    pub details: String,
}

#[derive(Serialize)]
pub struct IncidentDetail {
    pub id: String,
    pub category_code: String,
    pub category: CategoryDisplay,
    pub title: String,
    pub short_description: String,
    pub long_description: String,
    pub location: String,
    pub start_utc: String,
    pub end_utc: Option<String>,
    pub timeline: Vec<IncidentTimelineItem>,
    pub last_modified_at: Option<String>,
    pub last_modified_by: Option<AuditUser>,
}

#[derive(serde::Deserialize)]
pub struct IncidentListQuery {
    pub locale: Option<String>,
    pub q: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct IncidentTranslationsUpdateRequest {
    pub values: Vec<IncidentTranslationValue>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct IncidentTranslationValue {
    pub locale: String,
    pub field_key: String,
    pub field_value: String,
}

#[derive(serde::Serialize)]
pub struct IncidentTranslationMatrixRow {
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
pub struct IncidentTimelineEditItem {
    pub id: String,
    pub at_utc: Option<String>,
    pub sort_order: i32,
    pub fields: Vec<EditFieldValue>,
}

#[derive(serde::Serialize)]
pub struct IncidentEditData {
    pub id: String,
    pub category_id: String,
    pub start_utc: String,
    pub end_utc: Option<String>,
    pub locale: String,
    pub enabled_locales: Vec<String>,
    pub fields: Vec<EditFieldValue>,
    pub timeline: Vec<IncidentTimelineEditItem>,
}

#[derive(serde::Deserialize)]
pub struct IncidentTimelineSaveItem {
    pub id: String,
    pub at_utc: Option<String>,
    pub sort_order: i32,
    pub fields: HashMap<String, String>,
}

#[derive(serde::Deserialize)]
pub struct IncidentSaveRequest {
    pub id: String,
    pub category_id: String,
    pub start_utc: String,
    pub end_utc: Option<String>,
    pub locale: String,
    pub fields: HashMap<String, String>,
    #[serde(default)]
    pub replace_timeline: bool,
    pub timeline: Vec<IncidentTimelineSaveItem>,
}
