use std::collections::HashMap;

use serde::Serialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, ToSchema)]
pub struct CategoryDisplay {
    pub id: String,
    pub key: String,
    pub icon: String,
    pub color: String,
    pub label: String,
}

#[derive(Serialize, ToSchema)]
pub struct AuditUser {
    pub id: String,
    pub email: String,
}

#[derive(Serialize, ToSchema)]
pub struct IncidentListItem {
    pub key: String,
    pub category_id: String,
    pub category: CategoryDisplay,
    pub title: String,
    pub description: String,
    pub location: String,
    pub start_utc: String,
    pub end_utc: Option<String>,
    pub status_type: String,
    pub status_text: String,
    pub timeline: Vec<IncidentTimelineItem>,
}

#[derive(Serialize, ToSchema)]
pub struct IncidentTimelineItem {
    pub id: String,
    pub at_utc: Option<String>,
    pub title: String,
    pub details: String,
}

#[derive(Serialize, ToSchema)]
pub struct IncidentDetail {
    pub key: String,
    pub category_id: String,
    pub category: CategoryDisplay,
    pub title: String,
    pub description: String,
    pub location: String,
    pub start_utc: String,
    pub end_utc: Option<String>,
    pub status_type: String,
    pub status_text: String,
    pub timeline: Vec<IncidentTimelineItem>,
    pub last_modified_at: Option<String>,
    pub last_modified_by: Option<AuditUser>,
}

#[derive(serde::Deserialize, IntoParams)]
pub struct IncidentListQuery {
    pub locale: Option<String>,
    pub q: Option<String>,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct IncidentTranslationsUpdateRequest {
    pub values: Vec<IncidentTranslationValue>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, ToSchema)]
pub struct IncidentTranslationValue {
    pub locale: String,
    pub field_key: String,
    pub field_value: String,
}

#[derive(serde::Serialize, ToSchema)]
pub struct IncidentTranslationMatrixRow {
    pub field_key: String,
    pub locale: String,
    pub field_value: Option<String>,
}

pub use crate::common::validation::EditFieldValue;

#[derive(serde::Serialize, ToSchema)]
pub struct IncidentTimelineEditItem {
    pub id: String,
    pub at_utc: Option<String>,
    pub sort_order: i32,
    pub fields: Vec<EditFieldValue>,
}

#[derive(serde::Serialize, ToSchema)]
pub struct IncidentEditData {
    pub key: String,
    pub category_id: String,
    pub start_utc: String,
    pub end_utc: Option<String>,
    pub status_type: String,
    pub locale: String,
    pub enabled_locales: Vec<String>,
    pub fields: Vec<EditFieldValue>,
    pub timeline: Vec<IncidentTimelineEditItem>,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct IncidentTimelineSaveItem {
    pub id: String,
    pub at_utc: Option<String>,
    pub sort_order: i32,
    pub fields: HashMap<String, String>,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct IncidentSaveRequest {
    #[serde(default)]
    pub key: Option<String>,
    pub category_id: String,
    pub start_utc: String,
    pub end_utc: Option<String>,
    pub status_type: String,
    pub locale: String,
    pub fields: HashMap<String, String>,
    #[serde(default)]
    pub replace_timeline: bool,
    pub timeline: Vec<IncidentTimelineSaveItem>,
}

#[derive(serde::Serialize, ToSchema)]
pub struct CreatedKeyResponse {
    pub key: String,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct IncidentTimelineCreateRequest {
    pub at_utc: Option<String>,
    pub sort_order: i32,
    pub fields: HashMap<String, String>,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct IncidentTimelineUpdateRequest {
    pub at_utc: Option<String>,
    pub sort_order: i32,
    pub fields: HashMap<String, String>,
}
