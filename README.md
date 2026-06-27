# local-technique

Local-technique is a lightweight web app for co-owners to track building events, maintenance, and incidents with bilingual support and modern responsive navigation.

## Hosting

- Frontend: [GitHub Pages](https://local-technique.github.io/app)
- Backend: [Render.com](https://dashboard.render.com)
- Database: [Neon](https://console.neon.tech/app)
- Monitoring: [BetterStack](https://uptime.betterstack.com)
- Storage: [Cloudflare R2](https://cloudflare.com)

## OAuth clients configuration

- Google Console: https://console.cloud.google.com

## Environment variables

| Name                    | Description                                      | Default                        | Misc |
|-------------------------|--------------------------------------------------|--------------------------------|------|
| `APP_BASE_URL`          | Required. Backend public base URL.               | none                           | Local: `http://localhost:8080` |
| `FRONTEND_ORIGIN`       | Required. Allowed browser origin for CORS.       | none                           | Local: `http://localhost:5173` |
| `FRONTEND_BASE_URL`     | Optional. Frontend base URL for OAuth redirect.  | `FRONTEND_ORIGIN`              | Use when frontend runs under `/app`. |
| `GOOGLE_CLIENT_ID`      | Required. Google OAuth client ID.                | none                           | From Google Console. |
| `GOOGLE_CLIENT_SECRET`  | Required. Google OAuth client secret.            | none                           | Keep secret. |
| `MICROSOFT_CLIENT_ID`    | Required. Microsoft Entra ID app client ID.       | none                           | From Azure Portal app registration. |
| `MICROSOFT_CLIENT_SECRET`| Required. Microsoft Entra ID app client secret.   | none                           | Keep secret. |
| `COOKIE_KEY_BASE64`     | Required. Cookie key, base64 (>=64 decoded bytes). | none                         | PowerShell: `[Convert]::ToBase64String((1..64 \| ForEach-Object { Get-Random -Maximum 256 }))`; Bash: `openssl rand -base64 64` |
| `ACCESS_TOKEN_JWT_SECRET` | Required. JWT signing secret (`HS256`, 32 or 64 bytes random key are both acceptable). | none | PowerShell: `[Convert]::ToBase64String((1..64 \| ForEach-Object { Get-Random -Maximum 256 }))`; Bash: `openssl rand -base64 64` |
| `DATABASE_URL`          | Required. PostgreSQL DSN.                        | none                           | Local Docker: `postgres://postgres:example@localhost:5432/postgres` |
| `ADMIN_EMAILS`          | Optional. Comma-separated auto-admin emails.     | empty                          | Applied only for Google users. |
| `LISTEN_ADDR`           | Optional. Backend bind address.                  | `0.0.0.0:8080`                 | Example: `127.0.0.1:8080` |
| `RUST_LOG`              | Optional. Rust log level/filter.                 | `info`                         | Example: `debug` |
| `R2_ENDPOINT`           | Required. Cloudflare R2 S3-compatible endpoint.  | none                           | `https://<account-id>.r2.cloudflarestorage.com` |
| `R2_ATTACHMENTS_BUCKET` | Required. R2 bucket for file attachments.        | none                           | Auto-provisioned via Terraform. |

| `R2_ACCESS_KEY_ID`      | Required. R2 S3 credential access key.           | none                           | Keep secret. |
| `R2_SECRET_ACCESS_KEY`  | Required. R2 S3 credential secret key.           | none                           | Keep secret. |
