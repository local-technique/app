resource "render_web_service" "api" {
  name   = var.app_service_name != "" ? "${var.app_name}-${var.app_service_name}" : var.app_name
  plan   = var.deployment_plan
  region = var.deployment_region

  root_directory = var.app_root_directory
  start_command  = var.app_start_command

  runtime_source = {
    native_runtime = {
      auto_deploy   = true
      branch        = var.app_source_branch
      build_command = var.app_build_command
      repo_url      = local.app_source_repository_url
      runtime       = var.app_runtime
    }
  }

  env_vars = {
    TF_MANAGED_PLACEHOLDER = {
      value = "1"
    }
  }

  lifecycle {
    ignore_changes = [maintenance_mode, env_vars, plan]
  }
}
