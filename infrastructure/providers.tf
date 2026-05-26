provider "github" {
  owner = split("/", var.repository_full_name)[0]
  token = var.repository_token
}

provider "render" {
  owner_id = var.render_owner_id
}

provider "neon" {
  token = var.neon_api_token
}

provider "betteruptime" {}
