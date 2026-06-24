# Cloudflare R2 Storage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Provision two private Cloudflare R2 buckets (attachments + backups) using the Cloudflare Terraform provider.

**Architecture:** Add `cloudflare/cloudflare` provider (~> 5) alongside existing providers. Two R2 buckets created via `cloudflare_r2_bucket`, with CORS on the attachments bucket via `cloudflare_r2_bucket_cors` and 30-day lifecycle on backups via `cloudflare_r2_bucket_lifecycle`. R2 endpoint/bucket names injected as Render env vars so the backend can generate pre-signed URLs. Credentials passed to the app via the existing `app_secret_env_values` mechanism (no Terraform credential variables needed).

**Tech Stack:** Terraform 1.15, Cloudflare provider ~> 5, Cloudflare R2

---

### Task 1: Register Cloudflare provider and variable

**Files:**
- Modify: `infrastructure/versions.tf`
- Modify: `infrastructure/providers.tf`
- Modify: `infrastructure/variables.tf`

- [ ] **Step 1: Add cloudflare provider to versions.tf**

Insert the cloudflare block (in alphabetical order after `betteruptime` block):

```hcl
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 5"
    }
```

Expected result in `versions.tf`:

```hcl
    betteruptime = {
      source  = "BetterStackHQ/better-uptime"
      version = "~> 0.20"
    }

    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 5"
    }

    random = {
      source  = "hashicorp/random"
      version = "~> 3.7"
    }
```

- [ ] **Step 2: Add Cloudflare provider config to providers.tf**

Append after the `provider "betteruptime" {}` block:

```hcl
provider "cloudflare" {
  account_id = var.cloudflare_account_id
}
```

`CLOUDFLARE_API_TOKEN` is read automatically from the environment by the provider.

- [ ] **Step 3: Add cloudflare_account_id variable to variables.tf**

Append after the `database_role_name` variable (end of file, before the closing):

```hcl
variable "cloudflare_account_id" {
  description = "Cloudflare Account ID (32-char hex)."
  type        = string
}
```

---

### Task 2: Create storage.tf with buckets, CORS, and lifecycle

**Files:**
- Create: `infrastructure/storage.tf`

- [ ] **Step 1: Create storage.tf**

```hcl
resource "cloudflare_r2_bucket" "attachments" {
  account_id = var.cloudflare_account_id
  name       = "${var.app_name}-attachments"
  location   = "weur"
}

resource "cloudflare_r2_bucket" "backups" {
  account_id = var.cloudflare_account_id
  name       = "${var.app_name}-backups"
  location   = "weur"
}

resource "cloudflare_r2_bucket_cors" "attachments" {
  account_id  = var.cloudflare_account_id
  bucket_name = cloudflare_r2_bucket.attachments.name

  rules = [{
    allowed = {
      methods = ["GET", "PUT", "POST", "DELETE", "HEAD"]
      origins = [local.frontend_origin]
      headers = ["*"]
    }
    expose_headers  = ["ETag"]
    max_age_seconds = 3600
  }]
}

resource "cloudflare_r2_bucket_lifecycle" "backups" {
  account_id  = var.cloudflare_account_id
  bucket_name = cloudflare_r2_bucket.backups.name

  rules = [{
    id   = "expire-old-backups"
    conditions = {
      prefix = ""
    }
    enabled = true
    delete_objects_transition = {
      condition = {
        max_age = 2592000
        type    = "Age"
      }
    }
  }]
}
```

- [ ] **Step 2: Verify formatting**

Run: `terraform fmt storage.tf` in the `infrastructure/` directory.

---

### Task 3: Wire app env vars, tfvars example, and workflows

**Files:**
- Modify: `infrastructure/locals.tf`
- Modify: `infrastructure/terraform.tfvars.example`
- Modify: `.github/workflows/infrastructure-apply.yml`
- Modify: `.github/workflows/infrastructure-ci.yml`

- [ ] **Step 1: Add R2 env vars to locals.tf**

Add `R2_ENDPOINT`, `R2_ATTACHMENTS_BUCKET`, and `R2_BACKUPS_BUCKET` to the `app_env_vars_render` block in `locals.tf`. Insert after the `ACCESS_TOKEN_JWT_SECRET` entry:

```hcl
      R2_ENDPOINT = {
        value = "https://${var.cloudflare_account_id}.r2.cloudflarestorage.com"
      }
      R2_ATTACHMENTS_BUCKET = {
        value = cloudflare_r2_bucket.attachments.name
      }
      R2_BACKUPS_BUCKET = {
        value = cloudflare_r2_bucket.backups.name
      }
```

These are non-sensitive (bucket names and endpoint URL are public info once you know the account ID). The sensitive `R2_ACCESS_KEY_ID` and `R2_SECRET_ACCESS_KEY` must come from the existing `app_secret_env_values` mechanism (passed at CI time).

- [ ] **Step 2: Add cloudflare_account_id to terraform.tfvars.example**

Add after the `# Monitoring platform token` comment line:

```
# cloudflare_account_id   = "32-char-hex-from-cloudflare-dashboard"
```

- [ ] **Step 3: Add Cloudflare env vars to infrastructure-apply.yml**

Add to the `env:` block (after the `BETTERUPTIME_API_TOKEN` line):

```yaml
  CLOUDFLARE_API_TOKEN: ${{ secrets.CLOUDFLARE_API_TOKEN }}
  CLOUDFLARE_ACCOUNT_ID: ${{ vars.CLOUDFLARE_ACCOUNT_ID }}
  TF_VAR_cloudflare_account_id: ${{ vars.CLOUDFLARE_ACCOUNT_ID }}
```

Also add `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, and `R2_ACCOUNT_ID` to the `TF_VAR_app_secret_env_values` JSON map so the backend receives them at runtime:

```yaml
  TF_VAR_app_secret_env_values: |
    {
      "GOOGLE_CLIENT_ID": "${{ secrets.GOOGLE_CLIENT_ID }}",
      "GOOGLE_CLIENT_SECRET": "${{ secrets.GOOGLE_CLIENT_SECRET }}",
      "FACEBOOK_CLIENT_ID": "${{ secrets.FACEBOOK_CLIENT_ID }}",
      "FACEBOOK_CLIENT_SECRET": "${{ secrets.FACEBOOK_CLIENT_SECRET }}",
      "R2_ACCESS_KEY_ID": "${{ secrets.R2_ACCESS_KEY_ID }}",
      "R2_SECRET_ACCESS_KEY": "${{ secrets.R2_SECRET_ACCESS_KEY }}",
      "R2_ACCOUNT_ID": "${{ vars.CLOUDFLARE_ACCOUNT_ID }}"
    }
```

- [ ] **Step 4: Add Cloudflare env vars to infrastructure-ci.yml**

Same changes as Step 3, applied to `infrastructure-ci.yml`.

---

### Verification

After all tasks:

- [ ] Run `terraform fmt -check` or `make fmt` in `infrastructure/` to verify formatting
- [ ] Run `tflint` or `make lint` to check for issues
- [ ] Ensure `terraform init` succeeds (fetches cloudflare provider)
