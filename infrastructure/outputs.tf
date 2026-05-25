output "database_project_id" {
  description = "Database project identifier."
  value       = neon_project.app_db.id
}

output "database_branch_id" {
  description = "Database branch identifier used by the app."
  value       = neon_project.app_db.branch.id
}

output "app_database_name" {
  description = "Database name provisioned for the app."
  value       = neon_database.app_db.name
}

output "app_service_name" {
  description = "Application service name."
  value       = render_web_service.api.name
}

output "app_service_id" {
  description = "Application service identifier."
  value       = render_web_service.api.id
}

output "app_health_monitor_id" {
  description = "Better Stack monitor identifier for the app health endpoint."
  value       = betteruptime_monitor.api_health.id
}
