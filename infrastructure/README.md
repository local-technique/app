# Infrastructure

Terraform provisions a Neon database stack, a Render application stack, Better Stack monitoring, and repository-level Actions secret wiring.
Use the `Makefile` targets for both native and Docker execution.

## What This Configuration Provisions

| Component | Description |
| --- | --- |
| Database | Creates one Neon project, one branch, one role, and one database. Exposes the computed connection URI via `neon_connection_uri`. |
| App | Creates one Render web service from `local-technique/app` (branch `main` by default), rooted at `back/` with Rust native runtime. Injects non-sensitive env vars from `app_plain_env_vars`, injects `DATABASE_URL` from Neon, injects generated `COOKIE_KEY_BASE64` and `ACCESS_TOKEN_JWT_SECRET`, and sets `LISTEN_ADDR` from `app_port` unless overridden. Defaults in code target low-cost setup: `frankfurt` region, `starter` plan, and one instance. |
| Monitoring | Creates one Better Stack uptime monitor targeting `${render_web_service.api.url}/health`. |
| Repository | Creates/updates GitHub Actions secret `DATABASE_URL` in the current repository. |

## Inputs for CI

Create the following repository entries in GitHub (`Settings -> Secrets and variables -> Actions`).
Use **Secrets** for sensitive values and **Variables** for non-sensitive values.

### Secrets (required)

| Input | GitHub entry type | Name to create | What it does | How to supply / generate |
| --- | --- | --- | --- | --- |
| Render API token | secret | `RENDER_API_KEY` | Authenticates Render provider for web service management. | Create API key in Render account settings and store as GitHub Actions secret. |
| Better Stack API token | secret | `BETTERUPTIME_API_TOKEN` | Authenticates Better Stack provider for uptime monitor management. | Create API token in Better Stack Uptime settings and store as GitHub Actions secret. |
| Neon API token | secret | `TF_VAR_NEON_API_TOKEN` | Authenticates Neon resources (project/role/database/URI). | Create API key in Neon console and store as GitHub Actions secret. |
| GitHub repository token (local runs) | secret | `TF_VAR_REPOSITORY_TOKEN` | Authenticates GitHub provider when running Terraform locally. | Create PAT (or app token) with repository Actions secrets write access and export as `TF_VAR_repository_token` locally. CI uses `${{ github.token }}` instead. |

`COOKIE_KEY_BASE64` and `ACCESS_TOKEN_JWT_SECRET` are generated once by Terraform, persisted in state, and injected only into the Render service environment.

### Required vars (non-sensitive)

| Input | GitHub entry type | Name to create | What it does | How to supply |
| --- | --- | --- | --- | --- |
| Repository full name | inferred | none | Repository to manage in `owner/name` format. | Automatically set from `${{ github.repository }}` in CI. |
| Render owner id | variable | `RENDER_OWNER_ID` | Selects Render owner/team where resources are created (`usr-*` or `tea-*`). | Copy from Render dashboard URL for your user/team settings. |

### CI workflow token behavior

- `infrastructure-ci.yml` and `infrastructure-apply.yml` pass `${{ github.token }}` into `TF_VAR_repository_token`.
- Workflows set `permissions: actions: write` so the token can manage repository Actions secrets.
- Terraform config sets GitHub provider `owner` from `repository_full_name` to avoid the `/user` API call that `github.token` cannot access in Actions.
- Keep `TF_VAR_REPOSITORY_TOKEN` for local runs where `github.token` does not exist.

### Optional vars (use defaults)

Do not create GitHub entries for these unless you want to override defaults.

| Input | What it does | Default |
| --- | --- | --- |
| `database_project_name` | Neon project name. | `local-technique-db` |
| `database_region_id` | Neon region for the project (France). | `aws-eu-west-3` |
| `database_name` | Postgres database name in Neon. | `local-technique-db` |
| `database_role_name` | Postgres role used by app URI. | `app` |
| `app_name` | Render service name prefix. | `local-technique-backend` |
| `app_service_name` | Optional Render service name suffix. Empty keeps `app_name` as-is. | `""` |
| `app_source_repository` | GitHub source repository Render deploys from (`owner/repo` or full URL). | `local-technique/app` |
| `app_source_branch` | Git branch Render tracks. | `main` |
| `app_port` | Service HTTP port used in default `LISTEN_ADDR`. | `3000` |
| `deployment_region` | Deployment region for the app service. | `frankfurt` |
| `deployment_plan` | Service plan/tier for the app service. | `starter` |
| `app_root_directory` | Repository subdirectory used for builds. | `back` |
| `app_runtime` | Render native runtime. | `rust` |
| `app_build_command` | Build command for Render deploys. | `cargo build --release` |
| `app_start_command` | Start command for Render deploys. | `./target/release/back` |
| `render_owner_id` | Render owner/team id override. | `""` (use Render default owner context) |
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
