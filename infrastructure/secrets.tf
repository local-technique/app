resource "github_actions_secret" "database_url" {
  repository  = data.github_repository.repo.name
  secret_name = "DATABASE_URL"
  value       = data.neon_connection_uri.app_db.uri
}

resource "github_actions_variable" "backend_url" {
  repository    = data.github_repository.repo.name
  variable_name = "BACKEND_URL"
  value         = render_web_service.api.url
}

resource "github_actions_variable" "frontend_url" {
  repository    = data.github_repository.repo.name
  variable_name = "FRONTEND_URL"
  value         = "https://${split("/", var.repository_full_name)[0]}.github.io/${split("/", var.repository_full_name)[1]}"
}
