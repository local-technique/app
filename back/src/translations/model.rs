use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct TranslationMatrixRow {
    pub key_name: String,
    pub locale: String,
    pub value: Option<String>,
    pub is_missing: bool,
}

#[derive(Deserialize)]
pub struct BulkTranslationsUpdateRequest {
    pub values: Vec<BulkTranslationValue>,
}

#[derive(Deserialize)]
pub struct BulkTranslationValue {
    pub key_name: String,
    pub locale: String,
    pub value: String,
}
