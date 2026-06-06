use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct RoleDescriptor {
    pub code: &'static str,
    pub label_key: &'static str,
}

#[derive(Serialize)]
pub struct RolesResponse {
    pub roles: Vec<RoleDescriptor>,
}

#[derive(Debug, Deserialize)]
pub struct AdminUsersQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub search_email: Option<String>,
    pub role: Option<String>,
    pub sort: Option<String>,
    pub direction: Option<String>,
}

#[derive(Serialize)]
pub struct AdminUserItem {
    pub id: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub roles: Vec<String>,
}

#[derive(Serialize)]
pub struct AdminUsersResponse {
    pub items: Vec<AdminUserItem>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

#[derive(Deserialize)]
pub struct UpdateUserRolesRequest {
    pub roles: Vec<String>,
}

#[derive(Serialize)]
pub struct UpdateUserRolesResponse {
    pub id: String,
    pub roles: Vec<String>,
}
