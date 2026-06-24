terraform {
  required_version = "~> 1.15.0"

  backend "s3" {
    bucket = "copro-terraform-state"
    key    = "terraform.tfstate"
    region = "auto"
    endpoints = {
      s3 = "https://25f886c06cf71ca19c541154a63ec5c4.r2.cloudflarestorage.com"
    }
    skip_credentials_validation = true
    skip_region_validation      = true
    skip_requesting_account_id  = true
    skip_s3_checksum            = true
    use_path_style              = true
  }

  required_providers {
    github = {
      source  = "integrations/github"
      version = "~> 6.12"
    }

    render = {
      source  = "render-oss/render"
      version = "~> 1.8"
    }

    neon = {
      source  = "terraform-community-providers/neon"
      version = "~> 0.1.15"
    }

    betteruptime = {
      source  = "BetterStackHQ/better-uptime"
      version = "~> 0.21"
    }

    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 5"
    }

    random = {
      source  = "hashicorp/random"
      version = "~> 3.8"
    }
  }
}
