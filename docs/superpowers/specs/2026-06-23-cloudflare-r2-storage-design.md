# Cloudflare R2 Storage — Design Spec

## Overview

Add two private Cloudflare R2 buckets to the existing infrastructure:
- **attachments** — app files uploaded by users via pre-signed URLs
- **backups** — database backups uploaded via CI (manually/CI-triggered)

## Buckets

| Bucket | Name | Access | CORS | Lifecycle |
|--------|------|--------|------|-----------|
| Attachments | `${app_name}-attachments` | Private, pre-signed URLs | Yes (frontend origin) | None |
| Backups | `${app_name}-backups` | Private, CI-only | No | 30-day expiration (max_age: 2,592,000s) |

## Credentials (manual bootstrap)

Two credentials created once via Cloudflare Dashboard:

| Dashboard Item | Purpose | GitHub Secret |
|---------------|---------|--------------|
| API Token (`Workers R2 Storage Write`) | Terraform Cloudflare provider | `CLOUDFLARE_API_TOKEN` |
| R2 API Token (`Object Read & Write`) | App backend (pre-signed URLs, backups) | `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY` |

Cloudflare Account ID stored as GitHub `vars`: `CLOUDFLARE_ACCOUNT_ID`

## Terraform Changes

### `versions.tf` — Add provider
- `cloudflare/cloudflare` (~> 5)

### `providers.tf` — Configure
- `cloudflare` — reads `CLOUDFLARE_API_TOKEN` from env; `account_id` set via `var.cloudflare_account_id`

### `variables.tf` — New variable
- `cloudflare_account_id` (string) — provider + R2 bucket resources require it
- No R2 S3 credential variables needed in Terraform (only used at app runtime)

### `storage.tf` (new)
- `cloudflare_r2_bucket.attachments` — attachments bucket, location `weur`
- `cloudflare_r2_bucket.backups` — backups bucket, location `weur`
- `cloudflare_r2_bucket_cors.attachments` — CORS for attachments (origin: frontend URL, methods: GET/PUT/POST/DELETE/HEAD)
- `cloudflare_r2_bucket_lifecycle.backups` — expire objects after 30 days

### `locals.tf` — Add app env vars
- `R2_ENDPOINT` = `https://<account_id>.r2.cloudflarestorage.com`
- `R2_ATTACHMENTS_BUCKET` = bucket name
- `R2_BACKUPS_BUCKET` = bucket name

## Workflow Changes

Both `infrastructure-apply.yml` and `infrastructure-ci.yml` — add to `env`:

```yaml
CLOUDFLARE_API_TOKEN: ${{ secrets.CLOUDFLARE_API_TOKEN }}
CLOUDFLARE_ACCOUNT_ID: ${{ vars.CLOUDFLARE_ACCOUNT_ID }}
TF_VAR_cloudflare_account_id: ${{ vars.CLOUDFLARE_ACCOUNT_ID }}
```

`TF_VAR_app_secret_env_values` gets `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, and `R2_ACCOUNT_ID` added so the backend can generate pre-signed URLs — these come from GitHub secrets/vars, not Terraform variables.

## File Changes Summary

| File | Action |
|------|--------|
| `infrastructure/versions.tf` | Edit — add cloudflare provider |
| `infrastructure/providers.tf` | Edit — add Cloudflare provider config |
| `infrastructure/variables.tf` | Edit — add `cloudflare_account_id` |
| `infrastructure/storage.tf` | **New** — buckets, CORS, lifecycle |
| `infrastructure/locals.tf` | Edit — add R2 endpoint + bucket names |
| `infrastructure/terraform.tfvars.example` | Edit — document new var |
| `.github/workflows/infrastructure-apply.yml` | Edit — add env vars |
| `.github/workflows/infrastructure-ci.yml` | Edit — add env vars |
