terraform {
  required_version = "~> 1.15.0"

  required_providers {
    github = {
      source  = "integrations/github"
      version = "~> 6.0"
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
      version = "~> 0.20"
    }

    random = {
      source  = "hashicorp/random"
      version = "~> 3.7"
    }
  }
}
