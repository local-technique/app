locals {
  frontend_base_url = github_actions_variable.frontend_url.value

  frontend_origin = regex("^https?://[^/]+", local.frontend_base_url)

  app_env_defaults_merged = merge(
    {
      LISTEN_ADDR = "0.0.0.0:${var.app_port}"
    },
    var.app_plain_env_vars
  )

  app_env_vars_render = merge(
    { for key, value in local.app_env_defaults_merged : key => { value = value } },
    { for key, value in var.app_secret_env_values : key => { value = value } },
    {
      FRONTEND_BASE_URL = {
        value = local.frontend_base_url
      }
      FRONTEND_ORIGIN = {
        value = local.frontend_origin
      }
      DATABASE_URL = {
        value = data.neon_connection_uri.app_db.uri
      }
      COOKIE_KEY_BASE64 = {
        value = random_id.cookie_key_bytes.b64_std
      }
      ACCESS_TOKEN_JWT_SECRET = {
        value = random_id.access_token_jwt_secret_bytes.b64_std
      }
    }
  )

  app_source_repository_url = can(regex("^https?://", var.app_source_repository)) ? var.app_source_repository : "https://github.com/${var.app_source_repository}"

  app_healthcheck_url = var.app_healthcheck_url != "" ? var.app_healthcheck_url : "${render_web_service.api.url}/health"
}
