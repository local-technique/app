use utoipa::OpenApi;

mod http;
pub use http::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Categories
        crate::categories::http::list,
        crate::categories::http::admin_list,
        crate::categories::http::create,
        crate::categories::http::update,
        crate::categories::http::delete,
        // Incidents
        crate::incidents::http::list,
        crate::incidents::http::create,
        crate::incidents::http::detail,
        crate::incidents::http::update,
        crate::incidents::http::delete,
        crate::incidents::http::translations,
        crate::incidents::http::edit,
        crate::incidents::http::replace_translations,
        // Maintenances
        crate::maintenances::http::list,
        crate::maintenances::http::create,
        crate::maintenances::http::detail,
        crate::maintenances::http::update,
        crate::maintenances::http::delete,
        crate::maintenances::http::translations,
        crate::maintenances::http::edit,
        crate::maintenances::http::replace_translations,
        // Projects
        crate::projects::http::list,
        crate::projects::http::create,
        crate::projects::http::detail,
        crate::projects::http::update,
        crate::projects::http::delete,
        crate::projects::http::translations,
        crate::projects::http::edit,
        crate::projects::http::replace_translations,
    ),
    tags(
        (name = "categories", description = "Category management"),
        (name = "incidents", description = "Incident management"),
        (name = "maintenances", description = "Maintenance management"),
        (name = "projects", description = "Project management"),
    ),
)]
pub struct ApiDoc;
