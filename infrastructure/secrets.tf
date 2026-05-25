resource "github_actions_secret" "database_url" {
  repository  = data.github_repository.repo.name
  secret_name = "DATABASE_URL"
  value       = data.neon_connection_uri.app_db.uri
}
