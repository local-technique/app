# Microsoft Entra ID Auth (Replace Facebook)

**Date:** 2026-06-27

## Overview

Replace Facebook OAuth authentication with Microsoft Entra ID (personal accounts only) — same features: gather firstname, lastname, email; no forced admin role.

## Scope

Full removal of Facebook auth. No migration needed (no existing Facebook users). The provider name is `"microsoft"`.

---

## 1. Backend — Auth Model

### `back/src/auth/model.rs`

Add `Microsoft` variant to `Provider` enum, remove `Facebook`. Update `FromStr` and `Display` impls:

```rust
pub enum Provider {
    Google,
    Microsoft,
}
```

### `back/src/config.rs`

```rust
// Remove:
pub facebook_client_id: String,
pub facebook_client_secret: String,

// Add:
pub microsoft_client_id: String,
pub microsoft_client_secret: String,
```

### `back/.env`

Replace `FACEBOOK_CLIENT_ID=fake` / `FACEBOOK_CLIENT_SECRET=fake` with:

```
MICROSOFT_CLIENT_ID=fake
MICROSOFT_CLIENT_SECRET=fake
```

---

## 2. Backend — Auth Service

### `back/src/auth/service.rs`

Replace `facebook_exchange_and_email()` with `microsoft_exchange_and_email()` using the same raw-HTTP-call pattern.

**Endpoints:**

| Step | Endpoint |
|------|----------|
| Authorize | `https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize` |
| Token exchange | `POST https://login.microsoftonline.com/consumers/oauth2/v2.0/token` |
| User info | `GET https://graph.microsoft.com/v1.0/me?$select=displayName,givenName,surname,mail` |

**Scope:** `openid profile email User.Read`

**Response mapping:**

| Microsoft Graph field | `ProviderUser` field |
|-----------------------|---------------------|
| `givenName` | `first_name` |
| `surname` | `last_name` |
| `mail` | `email` |

Error if `mail` is absent (same guard as Facebook's email check).

**OAuth state flow:** Same as Facebook — stores state in in-memory `HashMap` with 10-minute TTL. Uses `reqwest` (already a dependency) for HTTP calls.

### `back/src/auth/http.rs`

Update `start_oauth()` and `oauth_callback()` — they dispatch on provider string, so no structural change needed.

### `back/src/auth/router.rs`

No changes needed — routes are `GET /auth/{provider}/start` and `GET /auth/{provider}/callback`, already generic.

---

## 3. Backend — Database

### New migration: `back/migrations/0025_microsoft_auth.sql`

```sql
-- Drop old constraint, add new one replacing facebook with microsoft
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_provider_check;
ALTER TABLE users ADD CONSTRAINT users_provider_check CHECK (provider IN ('google', 'microsoft'));
```

---

## 4. Frontend

### `front/src/auth/LoginPage.vue`

- Replace "Continue with Facebook" button with "Continue with Microsoft"
- Update href from `providerStartUrl("facebook", redirectPath)` to `providerStartUrl("microsoft", redirectPath)`
- Use a Microsoft logo/SVG icon (simple "M" or the Microsoft symbol)

### `front/src/auth/session.ts`

- Update `Provider` type: `"google" | "facebook"` → `"google" | "microsoft"`
- No other changes — `providerStartUrl()`, `exchangeCallbackCode()` etc. are provider-agnostic.

### No changes needed:

- `OAuthCallbackPage.vue` — generic, reads `?code=` from URL
- `router/index.ts` — routes are generic
- `authenticatedFetch.ts` — provider-agnostic

---

## 5. Infrastructure

### GitHub Actions workflows

**`.github/workflows/infrastructure-ci.yml`** and **`.github/workflows/infrastructure-apply.yml`**:

Replace:

```yaml
"FACEBOOK_CLIENT_ID": "${{ secrets.FACEBOOK_CLIENT_ID }}",
"FACEBOOK_CLIENT_SECRET": "${{ secrets.FACEBOOK_CLIENT_SECRET }}",
```

with:

```yaml
"MICROSOFT_CLIENT_ID": "${{ secrets.MICROSOFT_CLIENT_ID }}",
"MICROSOFT_CLIENT_SECRET": "${{ secrets.MICROSOFT_CLIENT_SECRET }}",
```

### GitHub Actions secrets

User must:
1. Add `MICROSOFT_CLIENT_ID` and `MICROSOFT_CLIENT_SECRET` as repo secrets
2. Remove `FACEBOOK_CLIENT_ID` and `FACEBOOK_CLIENT_SECRET` (optional)

### Documentation updates

- `infrastructure/terraform.tfvars.example` — replace Facebook entries with Microsoft
- `infrastructure/README.md` — replace Facebook entries with Microsoft
- `README.md` — replace Facebook env var documentation with Microsoft

---

## 6. Azure Portal Setup

1. Go to Azure Portal → Microsoft Entra ID → App registrations → New registration
2. Name: e.g. "CoPro Auth"
3. Supported account types: **"Personal Microsoft accounts only"**
4. Redirect URI (Web): `{APP_BASE_URL}/auth/microsoft/callback`
   - Dev: `http://localhost:8080/auth/microsoft/callback`
   - Prod: `https://<your-domain>/auth/microsoft/callback`
5. Create → note the **Application (client) ID** → this is `MICROSOFT_CLIENT_ID`
6. Certificates & secrets → New client secret → copy **Value** → this is `MICROSOFT_CLIENT_SECRET`
7. API permissions → `User.Read` (Microsoft Graph, delegated) should be pre-added; verify

---

## 7. Files to Delete

- Remove `facebook_start()` and `facebook_start_or_error()` functions from `service.rs`
- Remove `FacebookTokenResponse` and `FacebookUser` structs from `service.rs`
- Remove `Facebook` variant from `Provider` enum in `model.rs`
- Remove Facebook env var constants from `config.rs`

---

## 8. Data Flow (unchanged from Facebook)

1. User clicks "Continue with Microsoft" → `GET /auth/microsoft/start?redirect=...`
2. Backend creates OAuth state, redirects to Microsoft login page
3. User consents → Microsoft redirects to `GET /auth/microsoft/callback?code=...&state=...`
4. Backend validates state, exchanges code for access token
5. Backend calls Graph API `/me` to get profile (givenName, surname, mail)
6. User upserted in DB via `find_or_create_user()` by email
7. Auth session created, exchange code returned to frontend
8. Frontend exchanges code for JWT access token + refresh token
9. Subsequent requests use Bearer token via `authenticatedFetch.ts`
