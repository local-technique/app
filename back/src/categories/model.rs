use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct CategoryItem {
    pub id: String,
    pub key: String,
    pub icon: String,
    pub color: String,
    pub label: String,
    pub labels: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct CategoryListQuery {
    pub locale: Option<String>,
}

#[derive(Deserialize)]
pub struct CategoryCreateRequest {
    pub key: String,
    pub icon: String,
    pub color: String,
    pub labels: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct CategoryUpdateRequest {
    pub key: String,
    pub icon: String,
    pub color: String,
    pub labels: HashMap<String, String>,
}
