resource "neon_project" "app_db" {
  name      = var.database_project_name
  region_id = var.database_region_id

  branch = {
    name      = var.database_branch_name
    protected = false
  }

  pg_version = var.database_engine_major_version
}

resource "neon_role" "app_user" {
  name       = var.database_role_name
  branch_id  = neon_project.app_db.branch.id
  project_id = neon_project.app_db.id
}

resource "neon_database" "app_db" {
  name       = var.database_name
  owner_name = neon_role.app_user.name
  branch_id  = neon_project.app_db.branch.id
  project_id = neon_project.app_db.id
}

data "neon_connection_uri" "app_db" {
  project_id    = neon_project.app_db.id
  branch_id     = neon_project.app_db.branch.id
  database_name = neon_database.app_db.name
  role_name     = neon_role.app_user.name
}
