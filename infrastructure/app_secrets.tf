resource "random_id" "cookie_key_bytes" {
  byte_length = 64
}

resource "random_id" "access_token_jwt_secret_bytes" {
  byte_length = 64
}
