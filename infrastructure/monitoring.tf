resource "betteruptime_monitor" "api_health" {
  url                = local.app_healthcheck_url
  monitor_type       = "status"
  pronounceable_name = var.health_monitor_name
  follow_redirects   = true
  verify_ssl         = true
}
