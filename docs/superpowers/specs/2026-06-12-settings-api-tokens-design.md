# Settings & API Tokens Design

## Goal

Add a Settings view accessible from the sidebar navigation that displays the connected user's email and allows them to manage a single API token for programmatic access.

## Approach

Create a new `api_tokens` database table, a backend module with CRUD endpoints, and a Settings page in the frontend. Reuse the existing `/me` endpoint for displaying the user email. Branch the existing `Principal` extractor to accept either JWT (existing flow) or `lc_`-prefixed API tokens (verified via Argon2id against the database).

## Database Design

Add migration `0014_api_tokens.sql`.

```sql
CREATE TABLE IF NOT EXISTS api_tokens (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token_prefix TEXT NOT NULL,
  token_hash TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_used_at TIMESTAMPTZ
);

CREATE INDEX idx_api_tokens_user_id ON api_tokens(user_id);
```

- `user_id` — foreign key to the owning user. Enforced one-token-per-user at the application layer (unique constraint not needed since the insert/update logic checks and deletes existing).
- `token_prefix` — first 3 characters of the raw token, displayed in the UI for identification.
- `token_hash` — Argon2id hash of the raw token. The raw token is never stored.
- `created_at` — when the token was created.
- `last_used_at` — updated each time the token is verified in the `Principal` extractor.

## Backend Design

### New dependency

Add `argon2` crate to `Cargo.toml` with configuration: 19 MiB memory cost, 2 iterations, 1 degree of parallelism (Argon2id variant).

### New module: `back/src/api_tokens/`

- `model.rs` — `ApiToken` struct, request/response types:
  - `CreateTokenResponse { id, token_prefix, token_full, created_at }` — `token_full` returned only once
  - `TokenInfoResponse { id, token_prefix, created_at, last_used_at }` — returned by GET, no full token

- `repository.rs` — SQL functions:
  - `insert_token(db, user_id, token_prefix, token_hash) -> ApiToken`
  - `find_active_token(db, user_id) -> Option<ApiToken>` — check if user already has a token
  - `find_token_by_hash(db, hash) -> Option<(ApiToken, DbUser)>` — for Principal extractor lookup
  - `delete_token(db, user_id)` — revoke (delete row)
  - `update_last_used(db, token_id)` — update `last_used_at` to now

- `service.rs`:
  - `generate_token() -> (raw_token, token_prefix, token_hash)` — generates `lc_` + 32 random bytes (base64url no padding), hashes with Argon2id
  - `verify_token(raw_token, stored_hash) -> bool` — Argon2id verification
  - `create_token(db, user_id) -> CreateTokenResponse` — checks no existing token, generates, inserts, returns full token once
  - `get_token_info(db, user_id) -> Option<TokenInfoResponse>` — current token metadata
  - `renew_token(db, user_id) -> CreateTokenResponse` — deletes existing token (if any), generates new one, returns full token once
  - `revoke_token(db, user_id)` — deletes existing token row

- `http.rs` — handlers:
  - `POST /settings/token` — create or renew (if existing, deletes it first); returns full token
  - `GET /settings/token` — returns `TokenInfoResponse` or 404
  - `DELETE /settings/token` — revoke (204 No Content)

### Principal extractor change (`back/src/common/auth.rs`)

In `FromRequestParts` for `Principal`:

1. Extract `Authorization` header value.
2. If value starts with `"lc_"`:
   - Hash the token with Argon2id.
   - Look up matching `(ApiToken, DbUser)` by hash via `find_token_by_hash`.
   - If not found, reject with 401.
   - If found, update `last_used_at` on the token (fire-and-forget/spawn).
   - Return `Principal { user_id, roles }` from the associated user.
3. Else: existing JWT decode + session check flow.

The `lc_` prefix distinguishes API tokens from JWT tokens at the string level, avoiding any ambiguity.

### Routes (`back/src/app/router.rs`)

Add three new routes under a `/settings` scope:

| Method | Path | Handler | Auth |
|--------|------|---------|------|
| POST | `/settings/token` | `api_tokens::http::create_token` | Principal |
| GET | `/settings/token` | `api_tokens::http::get_token` | Principal |
| DELETE | `/settings/token` | `api_tokens::http::revoke_token` | Principal |

## Token Format

`lc_<base64url_no_padding(32_random_bytes)>`

- Total length: 3 (`lc_`) + 43 (32 bytes base64url encoded without padding) = 46 characters.
- 256 bits of entropy.
- Example: `lc_abc123...` (first 3 chars after prefix stored as `token_prefix`).

## Frontend Design

### Route (`front/src/router/index.ts`)

Add `/settings` with `requiresAuth: true` and no role restriction.

### Sidebar nav

Add a `Settings` link with the Lucide `Settings` icon at the bottom of the sidebar, near the language/theme controls. Only visible when authenticated.

### New page: `front/src/views/settings/SettingsPage.vue`

**Layout:**
- Top section: user email (fetched via GET `/me` on mount, or reused from session state).
- Bottom section: API token management card.

**Token states:**
1. **No token:** Show "No API token" message + "Generate token" button.
2. **Token exists:** Show:
   - Token prefix display (e.g., `lc_abc...`)
   - Creation date
   - Last used date (or "Never" if null)
   - Renew button (Lucide `RefreshCcw`)
   - Revoke button (Lucide `Trash2`, styled red)
3. **Token just generated:** Show a success banner with:
   - Full token in a monospace/code-style display
   - Copy button (Lucide `Copy`) — uses `navigator.clipboard.writeText()`
   - Dismiss notice: "This token won't be shown again"

**Copy interaction:**
- On click, copy full token to clipboard.
- Temporarily change icon to `Check` for 2 seconds as visual confirmation.

### API layer: `front/src/views/settings/api.ts`

```typescript
function getToken(): Promise<TokenInfoResponse | null>
function createToken(): Promise<CreateTokenResponse>
function revokeToken(): Promise<void>
```

Follows the existing repository/API pattern used by other domains.

### i18n

Add en/fr translation keys for all new UI strings in the existing `messages` object pattern:
- Page title ("Settings" / "Paramètres")
- Token section labels
- Button labels
- Success/error messages
- Copy confirmation ("Copied!" / "Copié !")

## Testing

### Backend

- Unit tests for token generation (format validation, prefix extraction).
- Unit tests for Argon2id hash/verify round-trip.
- Tests for `POST /settings/token` create and renew.
- Tests for `GET /settings/token` with and without token.
- Tests for `DELETE /settings/token` revoke.
- Tests for `Principal` extractor with valid/invalid `lc_` token.
- Note: Argon2id with 19 MiB / 2 iterations is already a lightweight config. Tests use the same config.

### Frontend

- Unit tests for Settings page token state rendering (no token, token exists, just generated).
- Unit tests for API layer functions.
- Follow existing frontend test patterns (vitest).

## Non-Goals

- Multiple tokens per user.
- Token expiry / TTL.
- Soft-delete / audit trail for revoked tokens.
- Rate limiting on token creation.
- Admin management of user tokens.
- Token scoping or permission subsetting — tokens inherit the full role set of the user.
