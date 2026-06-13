use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, ToSchema)]
pub struct CategoryItem {
    pub id: String,
    pub key: String,
    pub icon: String,
    pub color: String,
    pub label: String,
    pub labels: HashMap<String, String>,
}

#[derive(Deserialize, IntoParams)]
pub struct CategoryListQuery {
    pub locale: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct CategoryCreateRequest {
    pub key: String,
    pub icon: String,
    pub color: String,
    pub labels: HashMap<String, String>,
}

#[derive(Deserialize, ToSchema)]
pub struct CategoryUpdateRequest {
    pub key: String,
    pub icon: String,
    pub color: String,
    pub labels: HashMap<String, String>,
}
