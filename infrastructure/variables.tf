variable "repository_token" {
  description = "Token used to manage repository Actions secrets. In CI this can be the workflow GITHUB_TOKEN; for local runs set TF_VAR_repository_token to a PAT or app token."
  type        = string
  sensitive   = true
}

variable "repository_full_name" {
  description = "Repository in owner/name format where Actions secrets are managed."
  type        = string
}

variable "app_name" {
  description = "Application name."
  type        = string
  default     = "local-technique-backend"
}

variable "app_service_name" {
  description = "Optional service suffix appended to application name."
  type        = string
  default     = ""
}

variable "deployment_region" {
  description = "Region where the application service is deployed."
  type        = string
  default     = "frankfurt"
}

variable "deployment_plan" {
  description = "Service plan/tier for the application deployment."
  type        = string
  default     = "starter"
}

variable "app_root_directory" {
  description = "Repository subdirectory containing the backend app code for deployment builds."
  type        = string
  default     = "back"
}

variable "app_runtime" {
  description = "Runtime used by the deployment service."
  type        = string
  default     = "rust"
}

variable "app_build_command" {
  description = "Build command executed by the deployment service."
  type        = string
  default     = "cargo build --release"
}

variable "app_start_command" {
  description = "Start command executed by the deployment service."
  type        = string
  default     = "./target/release/back"
}

variable "app_source_repository" {
  description = "GitHub repository in owner/name format used for source deployment."
  type        = string
  default     = "local-technique/app"
}

variable "app_source_branch" {
  description = "Git branch tracked for source deployment."
  type        = string
  default     = "main"
}

variable "app_port" {
  description = "Application port used to build LISTEN_ADDR."
  type        = number
  default     = 3000
}

variable "render_owner_id" {
  description = "Optional Render owner/team id. Empty means Render default owner context."
  type        = string
  default     = ""
}

variable "health_monitor_name" {
  description = "Display name for the application health monitor."
  type        = string
  default     = "local-technique api health"
}

variable "app_healthcheck_url" {
  description = "Public healthcheck URL monitored by uptime checks. Empty uses the default deployment hostname and /health path."
  type        = string
  default     = ""
}

variable "app_plain_env_vars" {
  description = "Non-secret application environment variables as map of key to value."
  type        = map(string)
  default = {
    NODE_ENV = "production"
  }
}

variable "neon_api_token" {
  description = "Neon API token."
  type        = string
  sensitive   = true
}

variable "database_project_name" {
  description = "Database project name."
  type        = string
  default     = "local-technique-db"
}

variable "database_region_id" {
  description = "Database region id, for example aws-eu-west-3."
  type        = string
  default     = "aws-eu-west-3"
}

variable "database_branch_name" {
  description = "Database branch name for application workloads."
  type        = string
  default     = "main"
}

variable "database_engine_major_version" {
  description = "PostgreSQL major version for the database project."
  type        = number
  default     = 17
}

variable "database_name" {
  description = "Database name created for the app."
  type        = string
  default     = "local-technique-db"
}

variable "database_role_name" {
  description = "Role name created for application access."
  type        = string
  default     = "app"
}
