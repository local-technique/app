use serde::Serialize;

#[derive(Serialize)]
pub struct IncidentListItem {
    pub id: String,
    pub category_code: String,
    pub title: String,
    pub short_description: String,
    pub location: String,
    pub start_utc: String,
    pub end_utc: Option<String>,
}

#[derive(Serialize)]
pub struct IncidentTimelineItem {
    pub id: String,
    pub at_utc: String,
    pub title: String,
    pub details: String,
}

#[derive(Serialize)]
pub struct IncidentDetail {
    pub id: String,
    pub category_code: String,
    pub title: String,
    pub short_description: String,
    pub long_description: String,
    pub location: String,
    pub start_utc: String,
    pub end_utc: Option<String>,
    pub timeline: Vec<IncidentTimelineItem>,
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

#[derive(serde::Deserialize)]
pub struct IncidentUpsertRequest {
    pub id: String,
    pub category_code: String,
    pub start_utc: String,
    pub end_utc: Option<String>,
    pub translations: Vec<IncidentTranslationValue>,
    pub timeline: Vec<IncidentTimelineUpsertItem>,
}

#[derive(serde::Deserialize)]
pub struct IncidentTimelineUpsertItem {
    pub id: String,
    pub at_utc: String,
    pub sort_order: i32,
    pub translations: Vec<IncidentTranslationValue>,
}
