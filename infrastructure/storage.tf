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
    id = "expire-old-backups"
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
