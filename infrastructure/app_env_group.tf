resource "render_env_group" "app_runtime_urls" {
  name = var.app_service_name != "" ? "${var.app_name}-${var.app_service_name}-runtime-urls" : "${var.app_name}-runtime-urls"

  env_vars = merge(
    local.app_env_vars_render,
    {
      APP_BASE_URL = {
        value = render_web_service.api.url
      }
    }
  )
}

resource "render_env_group_link" "app_runtime_urls" {
  env_group_id = render_env_group.app_runtime_urls.id
  service_ids = [
    render_web_service.api.id
  ]

  lifecycle {
    replace_triggered_by = [
      render_env_group.app_runtime_urls
    ]
  }
}
