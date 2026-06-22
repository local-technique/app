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
pub struct ProjectTimelineItem {
    pub id: String,
    pub at_utc: Option<String>,
    pub title: String,
    pub details: String,
}

#[derive(Serialize, ToSchema)]
pub struct ProjectListItem {
    pub key: String,
    pub category_id: String,
    pub category: CategoryDisplay,
    pub title: String,
    pub description: String,
    pub start_utc: Option<String>,
    pub end_utc: Option<String>,
    pub status_type: String,
    pub status_text: String,
    pub timeline: Vec<ProjectTimelineItem>,
}

#[derive(Serialize, ToSchema)]
pub struct ProjectDetail {
    pub key: String,
    pub category_id: String,
    pub category: CategoryDisplay,
    pub title: String,
    pub description: String,
    pub start_utc: Option<String>,
    pub end_utc: Option<String>,
    pub status_type: String,
    pub status_text: String,
    pub timeline: Vec<ProjectTimelineItem>,
    pub last_modified_at: Option<String>,
    pub last_modified_by: Option<AuditUser>,
}

#[derive(serde::Deserialize, IntoParams)]
pub struct ProjectListQuery {
    pub locale: Option<String>,
    pub q: Option<String>,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct ProjectTranslationsUpdateRequest {
    pub values: Vec<ProjectTranslationValue>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq, ToSchema)]
pub struct ProjectTranslationValue {
    pub locale: String,
    pub field_key: String,
    pub field_value: String,
}

#[derive(serde::Serialize, ToSchema)]
pub struct ProjectTranslationMatrixRow {
    pub field_key: String,
    pub locale: String,
    pub field_value: Option<String>,
}

pub use crate::common::validation::EditFieldValue;

#[derive(serde::Serialize, ToSchema)]
pub struct ProjectTimelineEditItem {
    pub id: String,
    pub at_utc: Option<String>,
    pub sort_order: i32,
    pub fields: Vec<EditFieldValue>,
}

#[derive(serde::Serialize, ToSchema)]
pub struct ProjectEditData {
    pub key: String,
    pub category_id: String,
    pub start_utc: Option<String>,
    pub end_utc: Option<String>,
    pub status_type: String,
    pub locale: String,
    pub enabled_locales: Vec<String>,
    pub fields: Vec<EditFieldValue>,
    pub timeline: Vec<ProjectTimelineEditItem>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq, ToSchema)]
pub struct ProjectTimelineSaveItem {
    pub id: String,
    pub at_utc: Option<String>,
    pub sort_order: i32,
    pub fields: HashMap<String, String>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Eq, ToSchema)]
pub struct ProjectSaveRequest {
    #[serde(default)]
    pub key: Option<String>,
    pub category_id: String,
    pub start_utc: Option<String>,
    pub end_utc: Option<String>,
    pub status_type: String,
    pub locale: String,
    pub fields: HashMap<String, String>,
    #[serde(default)]
    pub replace_timeline: bool,
    pub timeline: Vec<ProjectTimelineSaveItem>,
}

#[derive(serde::Serialize, ToSchema)]
pub struct CreatedKeyResponse {
    pub key: String,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct ProjectTimelineCreateRequest {
    pub at_utc: Option<String>,
    pub sort_order: i32,
    pub fields: HashMap<String, String>,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct ProjectTimelineUpdateRequest {
    pub at_utc: Option<String>,
    pub sort_order: i32,
    pub fields: HashMap<String, String>,
}
