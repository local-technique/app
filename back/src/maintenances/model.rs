use serde::Serialize;

#[derive(Serialize)]
pub struct MaintenanceListItem {
    pub id: String,
    pub category_code: String,
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
    pub id: String,
    pub category_code: String,
    pub title: String,
    pub warning: String,
    pub short_description: String,
    pub long_description: String,
    pub location: String,
    pub start_utc: String,
    pub end_utc: Option<String>,
    pub notified_at_utc: Option<String>,
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

#[derive(serde::Deserialize)]
pub struct MaintenanceUpsertRequest {
    pub id: String,
    pub category_code: String,
    pub start_utc: String,
    pub end_utc: Option<String>,
    pub notified_at_utc: Option<String>,
    pub translations: Vec<MaintenanceTranslationValue>,
}
