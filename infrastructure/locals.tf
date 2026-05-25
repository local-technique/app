locals {
  app_env_defaults_merged = merge(
    {
      LISTEN_ADDR = "0.0.0.0:${var.app_port}"
    },
    var.app_plain_env_vars
  )

  app_env_vars_render = merge(
    { for key, value in local.app_env_defaults_merged : key => { value = value } },
    {
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
