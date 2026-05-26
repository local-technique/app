# Infrastructure

Terraform provisions a Neon database stack, a Render application stack, Better Stack monitoring, and repository-level Actions secret wiring.
Use the `Makefile` targets for both native and Docker execution.

## What This Configuration Provisions

| Component | Description |
| --- | --- |
| Database | Creates one Neon project, one branch, one role, and one database. Exposes the computed connection URI via `neon_connection_uri`. |
| App | Creates one Render web service from `local-technique/app` (branch `main` by default), rooted at `back/` with Rust native runtime. Injects non-sensitive env vars from `app_plain_env_vars`, injects `DATABASE_URL` from Neon, injects generated `COOKIE_KEY_BASE64` and `ACCESS_TOKEN_JWT_SECRET`, and sets `LISTEN_ADDR` from `app_port` unless overridden. Defaults in code target low-cost setup: `frankfurt` region, `free` plan, and one instance. |
| Repository | Creates/updates GitHub Actions secret `DATABASE_URL` and GitHub Actions variables `BACKEND_URL` and `FRONTEND_URL` in the current repository. |
| Monitoring | Creates one Better Stack uptime monitor targeting `${render_web_service.api.url}/health`. |

## Inputs for CI

Create the following repository entries in GitHub (`Settings -> Secrets and variables -> Actions`).
Use **Secrets** for sensitive values and **Variables** for non-sensitive values.

### Secrets (required)

| Input | GitHub entry type | Name to create | What it does | How to supply / generate |
| --- | --- | --- | --- | --- |
| GitHub App credentials | variable + secret | `CICD_AUTOMATION_GH_APP_CLIENT_ID`, `CICD_AUTOMATION_GH_APP_PK` | Generates a GitHub App installation token used by Terraform GitHub provider to manage repository Actions variables. | Create a GitHub App installed on this repo with Actions variables/secrets read-write, store app id as variable and private key as secret. |
| Render API token | secret | `RENDER_API_KEY` | Authenticates Render provider for web service management. | Create API key in Render account settings and store as GitHub Actions secret. |
| Better Stack API token | secret | `BETTERUPTIME_API_TOKEN` | Authenticates Better Stack provider for uptime monitor management. | Create API token in Better Stack Uptime settings and store as GitHub Actions secret. |
| Neon API token | secret | `TF_VAR_NEON_API_TOKEN` | Authenticates Neon resources (project/role/database/URI). | Create API key in Neon console and store as GitHub Actions secret. |
| OAuth/runtime app secrets | secret | `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `FACEBOOK_CLIENT_ID`, `FACEBOOK_CLIENT_SECRET` | Passed to Terraform as `TF_VAR_app_secret_env_values` and injected into Render runtime env vars. | Create each value as a GitHub Actions secret. |

`COOKIE_KEY_BASE64` and `ACCESS_TOKEN_JWT_SECRET` are generated once by Terraform, persisted in state, and injected only into the Render service environment.

### Required vars (non-sensitive)

| Input | GitHub entry type | Name to create | What it does | How to supply |
| --- | --- | --- | --- | --- |
| Repository full name | inferred | none | Repository to manage in `owner/name` format. | Automatically set from `${{ github.repository }}` in CI. |
| Render owner id | variable | `RENDER_OWNER_ID` | Owner/team id required by Render provider (`usr-*` or `tea-*`). | Use Render Owners API/UI to get a valid id. |
| Neon organization id | variable | `NEON_ORG_ID` | Organization id required to create Neon projects. | Copy from Neon organization settings page (`org-...`). |

### CI workflow token behavior

- `infrastructure-ci.yml` and `infrastructure-apply.yml` generate a GitHub App installation token using `actions/create-github-app-token@v3`.
- Workflows export that token to `TF_VAR_repository_token` for Terraform GitHub provider authentication.
- Workflows request `permissions: actions: write` for repository Actions variable management.
- Terraform config sets GitHub provider `owner` from `repository_full_name`.

### Optional vars (use defaults)

Do not create GitHub entries for these unless you want to override defaults.

| Input | What it does | Default |
| --- | --- | --- |
| `database_project_name` | Neon project name. | `local-technique-db` |
| `database_region_id` | Neon region for the project (closest available to France). | `aws-eu-west-2` |
| `database_history_retention_seconds` | Neon project history retention in seconds (respect org limits). | `21600` |
| `database_name` | Postgres database name in Neon. | `local-technique-db` |
| `database_role_name` | Postgres role used by app URI. | `app` |
| `app_name` | Render service name prefix. | `local-technique-backend` |
| `app_service_name` | Optional Render service name suffix. Empty keeps `app_name` as-is. | `""` |
| `app_source_repository` | GitHub source repository Render deploys from (`owner/repo` or full URL). | `local-technique/app` |
| `app_source_branch` | Git branch Render tracks. | `main` |
| `app_port` | Service HTTP port used in default `LISTEN_ADDR`. | `3000` |
| `deployment_region` | Deployment region for the app service. | `frankfurt` |
| `deployment_plan` | Service plan/tier for the app service. | `free` |
| `app_root_directory` | Repository subdirectory used for builds. | `back` |
| `app_runtime` | Render native runtime. | `rust` |
| `app_build_command` | Build command for Render deploys. | `cargo build --release` |
| `app_start_command` | Start command for Render deploys. | `./target/release/back` |
| `repository_full_name` | Repository full name (`owner/repo`) for Actions variable management. | none |
| `repository_token` | Token for GitHub provider variable management. | none |
| `render_owner_id` | Render owner/team id required by provider. | none |
| `health_monitor_name` | Health monitor display name. | `local-technique api health` |
| `app_healthcheck_url` | Healthcheck URL to monitor. Empty auto-builds URL from service name. | `""` |
| `app_plain_env_vars` | Non-sensitive env var map for runtime injection. | `{ NODE_ENV = "production" }` |
| `database_branch_name` | Neon branch used for role/database resources. | `main` |
| `database_engine_major_version` | PostgreSQL major version for Neon project. | `17` |

## State Strategy

- CI and apply use local Terraform state file (`terraform.tfstate`) restored/saved through GitHub Actions cache.
- No AWS backend is configured in this repository.
- Cache-backed state is simple and works for this setup, but it is not equivalent to a fully durable remote backend.

## Workflows

| Workflow | File | Trigger | Behavior |
| --- | --- | --- | --- |
| CI | `.github/workflows/infrastructure-ci.yml` | Pull requests touching infra files | Installs latest Terraform + TFLint, runs lint and plan, posts plan output as sticky PR comment. |
| Apply | `.github/workflows/infrastructure-apply.yml` | Push to `main` touching infra files | Installs latest Terraform, restores cached state, runs apply only (`init` + `apply`), then saves updated state cache. |

## Make Targets

| Context | Targets |
| --- | --- |
| native | `make fmt`, `make lint`, `make init`, `make plan`, `make show-plan`, `make apply` |
| docker | `make fmt-docker`, `make lint-docker`, `make init-docker`, `make plan-docker`, `make show-plan-docker`, `make apply-docker` |

## Local Docker Usage

```sh
cp terraform.tfvars.example terraform.tfvars

make init-docker

make validate-docker
make plan-docker
make apply-docker
```
