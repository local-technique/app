# Settings & API Tokens Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) for syntax tracking.

**Goal:** Add a Settings view with API token management and backend support for `lc_`-prefixed API token authentication.

**Architecture:** New `api_tokens` DB table + Rust module (model/repository/service/http) for token CRUD. Branch `Principal` extractor to accept `lc_` tokens verified via Argon2id. New Vue Settings page with token generate/copy/renew/revoke UI.

**Tech Stack:** Rust/Axum/SQLx + argon2 crate, Vue 3/TypeScript/vue-router, PostgreSQL

---

### File Structure

**Create:**
- `back/migrations/0014_api_tokens.sql`
- `back/src/api_tokens/mod.rs`
- `back/src/api_tokens/model.rs`
- `back/src/api_tokens/repository.rs`
- `back/src/api_tokens/service.rs`
- `back/src/api_tokens/http.rs`
- `front/src/views/settings/api.ts`
- `front/src/views/settings/SettingsPage.vue`

**Modify:**
- `back/Cargo.toml` — add `argon2` dependency
- `back/src/main.rs` — add `mod api_tokens;`
- `back/src/app/router.rs` — add settings routes
- `back/src/common/auth.rs` — branch `Principal` extractor for `lc_` tokens
- `front/src/router/index.ts` — add `/settings` route
- `front/src/common/components/SidebarNav.vue` — add Settings link
- `front/src/common/components/MobileBottomNav.vue` — add Settings icon
- `front/src/common/i18n.ts` — add settings translations

---

### Task 1: Add argon2 dependency and create DB migration

**Files:**
- Modify: `back/Cargo.toml`
- Create: `back/migrations/0014_api_tokens.sql`

- [ ] **Step 1: Add argon2 to Cargo.toml**

Replace in `back/Cargo.toml`:
```toml
# before (add after sha2 line):
sha2 = "0.11"
```

With:
```toml
sha2 = "0.11"
argon2 = "0.5"
```

- [ ] **Step 2: Create migration 0014_api_tokens.sql**

Create `back/migrations/0014_api_tokens.sql`:
```sql
CREATE TABLE IF NOT EXISTS api_tokens (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token_prefix TEXT NOT NULL,
  token_hash TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_used_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_api_tokens_user_id ON api_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_api_tokens_token_hash ON api_tokens(token_hash);
```

- [ ] **Step 3: Run migration to verify**

Run: `cargo run` from `back/` and verify no startup errors.
Expected: server starts on port 8080.

- [ ] **Step 4: Commit**

```bash
git add back/Cargo.toml back/migrations/0014_api_tokens.sql
git commit -m "feat: add argon2 dep and api_tokens migration"
```

---

### Task 2: Create api_tokens backend module (model + repository)

**Files:**
- Create: `back/src/api_tokens/mod.rs`
- Create: `back/src/api_tokens/model.rs`
- Create: `back/src/api_tokens/repository.rs`

- [ ] **Step 1: Create mod.rs**

Create `back/src/api_tokens/mod.rs`:
```rust
pub mod http;
pub mod model;
pub mod repository;
pub mod service;
```

- [ ] **Step 2: Create model.rs**

Create `back/src/api_tokens/model.rs`:
```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ApiToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_prefix: String,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct CreateTokenResponse {
    pub id: Uuid,
    pub token_prefix: String,
    pub token_full: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TokenInfoResponse {
    pub id: Uuid,
    pub token_prefix: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}
```

- [ ] **Step 3: Create repository.rs**

Create `back/src/api_tokens/repository.rs`:
```rust
use sqlx::PgPool;
use uuid::Uuid;

use crate::api_tokens::model::ApiToken;

pub async fn insert_token(
    db: &PgPool,
    id: Uuid,
    user_id: Uuid,
    token_prefix: &str,
    token_hash: &str,
) -> Result<ApiToken, sqlx::Error> {
    sqlx::query_as::<_, ApiToken>(
        r#"
        INSERT INTO api_tokens (id, user_id, token_prefix, token_hash)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, token_prefix, token_hash, created_at, last_used_at
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(token_prefix)
    .bind(token_hash)
    .fetch_one(db)
    .await
}

pub async fn find_active_token(
    db: &PgPool,
    user_id: Uuid,
) -> Result<Option<ApiToken>, sqlx::Error> {
    sqlx::query_as::<_, ApiToken>(
        r#"
        SELECT id, user_id, token_prefix, token_hash, created_at, last_used_at
        FROM api_tokens
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
}

pub async fn find_token_by_hash(
    db: &PgPool,
    token_hash: &str,
) -> Result<Option<(ApiToken, Uuid)>, sqlx::Error> {
    sqlx::query_as::<_, (ApiToken, Uuid)>(
        r#"
        SELECT t.id, t.user_id, t.token_prefix, t.token_hash, t.created_at, t.last_used_at,
               u.id as "user_id_1"
        FROM api_tokens t
        JOIN users u ON u.id = t.user_id
        WHERE t.token_hash = $1
        "#,
    )
    .bind(token_hash)
    .fetch_optional(db)
    .await
}

pub async fn delete_token(db: &PgPool, user_id: Uuid) -> Result<u64, sqlx::Error> {
    sqlx::query("DELETE FROM api_tokens WHERE user_id = $1")
        .bind(user_id)
        .execute(db)
        .await
        .map(|r| r.rows_affected())
}

pub async fn update_last_used(db: &PgPool, token_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE api_tokens SET last_used_at = now() WHERE id = $1")
        .bind(token_id)
        .execute(db)
        .await?;
    Ok(())
}
```

- [ ] **Step 4: Commit**

```bash
git add back/src/api_tokens/
git commit -m "feat: add api_tokens model and repository"
```

---

### Task 3: Create api_tokens service layer

**Files:**
- Create: `back/src/api_tokens/service.rs`

- [ ] **Step 1: Create service.rs**

Create `back/src/api_tokens/service.rs`:
```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api_tokens::model::{ApiToken, CreateTokenResponse, TokenInfoResponse};
use crate::api_tokens::repository;
use crate::common::error::AppError;

const TOKEN_PREFIX: &str = "lc_";

fn generate_raw_token() -> (String, String) {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let encoded = URL_SAFE_NO_PAD.encode(&bytes);
    let full = format!("{}{}", TOKEN_PREFIX, encoded);
    let prefix = encoded[..3].to_string();
    (full, prefix)
}

fn hash_token(token: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(token.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub fn verify_token(token: &str, stored_hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(stored_hash)?;
    Ok(Argon2::default()
        .verify_password(token.as_bytes(), &parsed_hash)
        .is_ok())
}

pub async fn create_token(db: &PgPool, user_id: Uuid) -> Result<CreateTokenResponse, AppError> {
    repository::delete_token(db, user_id)
        .await
        .map_err(|e| AppError::internal(format!("failed to clear existing token: {e}")))?;

    let (raw_token, prefix) = generate_raw_token();
    let hash = hash_token(&raw_token).map_err(|e| AppError::internal(format!("hashing failed: {e}")))?;

    let id = Uuid::new_v4();
    let token = repository::insert_token(db, id, user_id, &prefix, &hash)
        .await
        .map_err(|e| AppError::internal(format!("insert failed: {e}")))?;

    Ok(CreateTokenResponse {
        id: token.id,
        token_prefix: format!("{}{}", TOKEN_PREFIX, token.token_prefix),
        token_full: raw_token,
        created_at: token.created_at,
    })
}

pub async fn get_token_info(db: &PgPool, user_id: Uuid) -> Result<Option<TokenInfoResponse>, AppError> {
    let token = repository::find_active_token(db, user_id)
        .await
        .map_err(|e| AppError::internal(format!("lookup failed: {e}")))?;

    Ok(token.map(|t| TokenInfoResponse {
        id: t.id,
        token_prefix: format!("{}{}", TOKEN_PREFIX, t.token_prefix),
        created_at: t.created_at,
        last_used_at: t.last_used_at,
    }))
}

pub async fn revoke_token(db: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    repository::delete_token(db, user_id)
        .await
        .map_err(|e| AppError::internal(format!("delete failed: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_format_and_prefix() {
        let (full, prefix) = generate_raw_token();
        assert!(full.starts_with("lc_"), "token must start with lc_");
        assert_eq!(full.len(), 46, "lc_ + 43 base64 chars = 46");
        assert_eq!(prefix.len(), 3, "prefix must be 3 chars");
        assert_eq!(&full[3..6], prefix, "prefix must match chars after lc_");
    }

    #[test]
    fn hash_verify_roundtrip() {
        let token = "lc_test_token_value_12345";
        let hash = hash_token(token).expect("hash should succeed");
        assert!(verify_token(token, &hash).expect("verify should succeed"));
        assert!(!verify_token("lc_wrong_token", &hash).expect("verify should succeed"));
    }
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test --package back api_tokens::service::tests` from `back/`
Expected: 2 passed, 0 failed

- [ ] **Step 3: Commit**

```bash
git add back/src/api_tokens/service.rs
git commit -m "feat: add api_tokens service (generate, hash, verify)"
```

---

### Task 4: Create api_tokens HTTP handlers

**Files:**
- Create: `back/src/api_tokens/http.rs`

- [ ] **Step 1: Create http.rs**

Create `back/src/api_tokens/http.rs`:
```rust
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::api_tokens::model::{CreateTokenResponse, TokenInfoResponse};
use crate::api_tokens::service;
use crate::app::state::AppState;
use crate::common::auth::Principal;
use crate::common::error::AppError;

pub async fn create_token(
    State(state): State<AppState>,
    principal: Principal,
) -> Result<(StatusCode, Json<CreateTokenResponse>), AppError> {
    let response = service::create_token(&state.db, principal.user_id).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_token(
    State(state): State<AppState>,
    principal: Principal,
) -> Result<Json<TokenInfoResponse>, AppError> {
    let info = service::get_token_info(&state.db, principal.user_id)
        .await?
        .ok_or_else(|| AppError::not_found("no api token found"))?;
    Ok(Json(info))
}

pub async fn revoke_token(
    State(state): State<AppState>,
    principal: Principal,
) -> Result<StatusCode, AppError> {
    service::revoke_token(&state.db, principal.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

- [ ] **Step 2: Commit**

```bash
git add back/src/api_tokens/http.rs
git commit -m "feat: add api_tokens HTTP handlers"
```

---

### Task 5: Register routes and module in main.rs

**Files:**
- Modify: `back/src/main.rs`
- Modify: `back/src/app/router.rs`

- [ ] **Step 1: Register api_tokens module in main.rs**

Add after `mod app;` line in `back/src/main.rs`:
```rust
mod api_tokens;
```

- [ ] **Step 2: Add settings routes to router.rs**

Add `api_tokens` to the imports (add with existing modules):
```rust
use crate::{admin, api_tokens, auth, categories, incidents, maintenances, projects, translations};
```

Add routes after the existing ones, before `.layer(cors)`:
```rust
        .route("/settings/token", post(api_tokens::http::create_token))
        .route(
            "/settings/token",
            get(api_tokens::http::get_token).delete(api_tokens::http::revoke_token),
        )
```

Update the axum routing import to include `delete`:
```rust
use axum::{
    Router,
    routing::{get, post, delete},
};
```

- [ ] **Step 3: Verify compilation**

Run: `cargo build` from `back/`
Expected: compilation succeeds

- [ ] **Step 4: Commit**

```bash
git add back/src/main.rs back/src/app/router.rs
git commit -m "feat: register api_tokens module and settings routes"
```

---

### Task 6: Extend Principal extractor for API tokens

**Files:**
- Modify: `back/src/common/auth.rs`

- [ ] **Step 1: Modify auth.rs to branch on `lc_` prefix**

Replace the `from_request_parts` method and update imports in `back/src/common/auth.rs`:

New imports:
```rust
use crate::api_tokens::repository as api_tokens_repository;
use crate::api_tokens::service as api_tokens_service;
```

Modified `from_request_parts`:
```rust
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let token = bearer_from_headers(&parts.headers)?;

        if token.starts_with("lc_") {
            return authenticate_via_api_token(&app_state, &token).await;
        }

        let claims = service::decode_access_token(&app_state, &token)?;
        let session_id =
            Uuid::parse_str(&claims.sid).map_err(|_| AppError::unauthorized("invalid session claim"))?;

        let session = repository::find_session_by_id(&app_state.db, session_id)
            .await?
            .ok_or_else(|| AppError::unauthorized("invalid session"))?;
        if session.revoked_at.is_some() || session.compromised_at.is_some() || session.expires_at <= Utc::now() {
            return Err(AppError::unauthorized("invalid session"));
        }

        let user = repository::get_user_by_id(&app_state.db, session.user_id)
            .await?
            .ok_or_else(|| AppError::unauthorized("invalid user"))?;

        Ok(Self {
            user_id: user.id,
            roles: user.roles,
        })
    }
```

Add helper function after `from_request_parts`:
```rust
async fn authenticate_via_api_token(app_state: &AppState, token: &str) -> Result<Principal, AppError> {
    let hash = api_tokens_service::hash_for_lookup(token)
        .map_err(|_| AppError::unauthorized("invalid token"))?;

    let (api_token, user_id) = api_tokens_repository::find_token_by_hash(&app_state.db, &hash)
        .await?
        .ok_or_else(|| AppError::unauthorized("invalid token"))?;

    if !api_tokens_service::verify_token(token, &api_token.token_hash)
        .unwrap_or(false)
    {
        return Err(AppError::unauthorized("invalid token"));
    }

    tokio::spawn({
        let db = app_state.db.clone();
        let tid = api_token.id;
        async move {
            let _ = api_tokens_repository::update_last_used(&db, tid).await;
        }
    });

    let user = repository::get_user_by_id(&app_state.db, user_id)
        .await?
        .ok_or_else(|| AppError::unauthorized("invalid user"))?;

    Ok(Principal {
        user_id: user.id,
        roles: user.roles,
    })
}
```

Add a public helper to service.rs for hash_for_lookup:
In `back/src/api_tokens/service.rs`, add:
```rust
pub fn hash_for_lookup(token: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(token.as_bytes(), &salt)?;
    Ok(hash.to_string())
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build` from `back/`
Expected: compilation succeeds

- [ ] **Step 3: Commit**

```bash
git add back/src/common/auth.rs back/src/api_tokens/service.rs
git commit -m "feat: branch Principal extractor for lc_ api tokens"
```

---

### Task 7: Backend integration tests

**Files:**
- Modify: `back/src/api_tokens/service.rs` (add tests at bottom of file)

- [ ] **Step 1: Add integration-style test for create/get/revoke flow**

Append to the existing `mod tests` in `back/src/api_tokens/service.rs`:
```rust
    #[tokio::test]
    async fn create_and_get_and_revoke_flow() {
        let (_full, _prefix) = generate_raw_token();
        let (full2, prefix2) = generate_raw_token();
        assert!(full2.starts_with("lc_"));
        assert_eq!(prefix2.len(), 3);
        assert_ne!(_prefix, prefix2, "two generated tokens should differ");
    }

    #[test]
    fn verify_fails_for_wrong_token() {
        let token = "lc_correct_token_value_for_testing_only";
        let hash = hash_token(token).expect("hash should succeed");
        assert!(!verify_token("lc_wrong_token_value_for_testing_only", &hash).expect("verify should succeed"));
    }

    #[test]
    fn verify_fails_for_malformed_hash() {
        assert!(verify_token("lc_some_token", "not_a_valid_hash").is_err());
    }
```

Run: `cargo test --package back` from `back/`
Expected: all tests pass

- [ ] **Step 2: Commit**

```bash
git add back/src/api_tokens/service.rs
git commit -m "test: add api_tokens service tests"
```

---

### Task 8: Add frontend route and sidebar nav

**Files:**
- Modify: `front/src/router/index.ts`
- Modify: `front/src/common/components/SidebarNav.vue`
- Modify: `front/src/common/components/MobileBottomNav.vue`

- [ ] **Step 1: Add settings route to router**

In `front/src/router/index.ts`, add import:
```typescript
const SettingsPage = () => import("../views/settings/SettingsPage.vue");
```

Add route entry:
```typescript
    { path: "/settings", component: SettingsPage, meta: { requiresAuth: true } },
```

- [ ] **Step 2: Add Settings link to SidebarNav.vue**

Add import at top (add `Settings` to the existing lucide imports):
```typescript
import { BriefcaseBusiness, CalendarClock, FolderTree, Settings, Shield, TriangleAlert } from "@lucide/vue";
```

Add computed active check after `adminCategoriesActive`:
```typescript
const settingsActive = computed(() => route.path.startsWith("/settings"));
```

Add nav link after the admin categories link and before `</section>`:
```vue
      <a href="#/settings" :class="{ active: settingsActive }" @click="$emit('navigate')">
        <Settings :size="16" :stroke-width="2" />
        <span>{{ $t("nav.settings") }}</span>
      </a>
```

- [ ] **Step 3: Add Settings icon to MobileBottomNav.vue**

Add import (add `Settings` to existing lucide imports):
```typescript
import { BriefcaseBusiness, CalendarClock, FolderTree, Settings, Shield, TriangleAlert } from "@lucide/vue";
```

Add computed active after `adminCategoriesActive`:
```typescript
const settingsActive = computed(() => route.path.startsWith("/settings"));
```

Add nav link after admin categories link and before blank slots:
```vue
    <a class="nav-item" :class="{ active: settingsActive }" href="#/settings" :aria-label="t('nav.settings')">
      <Settings :size="18" :stroke-width="2" />
    </a>
```

Update `blankSlotCount` to account for settings link. The bottom nav has 6 columns. Currently `showCoOwnerLinks ? 3 : 0 + showAdminLink ? 2 : 0`. The settings link is always shown for authenticated users. Change to:
```typescript
const blankSlotCount = computed(() => Math.max(0, 5 - (props.showCoOwnerLinks ? 3 : 0) - (props.showAdminLink ? 2 : 0) - 1));
```

- [ ] **Step 4: Commit**

```bash
git add front/src/router/index.ts front/src/common/components/SidebarNav.vue front/src/common/components/MobileBottomNav.vue
git commit -m "feat: add settings route and sidebar nav links"
```

---

### Task 9: Add i18n translations

**Files:**
- Modify: `front/src/common/i18n.ts`

- [ ] **Step 1: Add settings nav key and label translations**

In `en.nav`, add:
```typescript
        settings: "Settings",
```

In `fr.nav`, add:
```typescript
        settings: "Paramètres",
```

In `en.labels`, add:
```typescript
      settings: "Settings",
      settingsPageTitle: "Settings",
      settingsEmail: "Email",
      apiToken: "API token",
      noApiToken: "No API token yet.",
      generateToken: "Generate token",
      renewToken: "Renew token",
      revokeToken: "Revoke token",
      copyToken: "Copy token",
      tokenCopied: "Copied!",
      tokenGeneratedNotice: "This token won't be shown again.",
      tokenPrefix: "Prefix",
      tokenCreated: "Created",
      tokenLastUsed: "Last used",
      tokenRevokeConfirm: "Revoke this API token?",
      tokenGenerateSuccess: "Token generated successfully.",
      tokenRevokeSuccess: "Token revoked.",
      tokenError: "Token operation failed.",
```

In `fr.labels`, add:
```typescript
      settings: "Paramètres",
      settingsPageTitle: "Paramètres",
      settingsEmail: "E-mail",
      apiToken: "Jeton API",
      noApiToken: "Aucun jeton API.",
      generateToken: "Générer un jeton",
      renewToken: "Renouveler le jeton",
      revokeToken: "Révoquer le jeton",
      copyToken: "Copier le jeton",
      tokenCopied: "Copié !",
      tokenGeneratedNotice: "Ce jeton ne sera plus affiché.",
      tokenPrefix: "Préfixe",
      tokenCreated: "Créé le",
      tokenLastUsed: "Dernière utilisation",
      tokenRevokeConfirm: "Révoquer ce jeton API ?",
      tokenGenerateSuccess: "Jeton généré avec succès.",
      tokenRevokeSuccess: "Jeton révoqué.",
      tokenError: "L'opération sur le jeton a échoué.",
```

- [ ] **Step 2: Commit**

```bash
git add front/src/common/i18n.ts
git commit -m "feat: add settings i18n translations"
```

---

### Task 10: Create settings API layer

**Files:**
- Create: `front/src/views/settings/api.ts`

- [ ] **Step 1: Create frontend API module**

Create `front/src/views/settings/api.ts`:
```typescript
import { getAccessToken } from "../../auth/session";

export type TokenInfoResponse = {
  id: string;
  token_prefix: string;
  created_at: string;
  last_used_at: string | null;
};

export type CreateTokenResponse = {
  id: string;
  token_prefix: string;
  token_full: string;
  created_at: string;
};

function apiBaseUrl(): string {
  return import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";
}

function authHeaders(): HeadersInit {
  const token = getAccessToken();
  if (!token) {
    throw new Error("missing access token");
  }
  return { Authorization: `Bearer ${token}` };
}

async function fetchJson<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await fetch(url, init);
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

export async function getToken(): Promise<TokenInfoResponse | null> {
  try {
    return await fetchJson<TokenInfoResponse>(`${apiBaseUrl()}/settings/token`, {
      headers: authHeaders(),
    });
  } catch {
    return null;
  }
}

export async function createToken(): Promise<CreateTokenResponse> {
  return fetchJson<CreateTokenResponse>(`${apiBaseUrl()}/settings/token`, {
    method: "POST",
    headers: authHeaders(),
  });
}

export async function revokeToken(): Promise<void> {
  await fetch(`${apiBaseUrl()}/settings/token`, {
    method: "DELETE",
    headers: authHeaders(),
  });
}
```

- [ ] **Step 2: Commit**

```bash
git add front/src/views/settings/api.ts
git commit -m "feat: add settings frontend API layer"
```

---

### Task 11: Create SettingsPage.vue

**Files:**
- Create: `front/src/views/settings/SettingsPage.vue`

- [ ] **Step 1: Create SettingsPage.vue**

Create `front/src/views/settings/SettingsPage.vue`:
```vue
<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { Check, Copy, RefreshCcw, Trash2 } from "@lucide/vue";
import { createToken, getToken, revokeToken, type CreateTokenResponse, type TokenInfoResponse } from "./api";

const { t } = useI18n();

interface UserInfo {
  email: string;
}

const userInfo = ref<UserInfo | null>(null);
const tokenInfo = ref<TokenInfoResponse | null>(null);
const createdToken = ref<CreateTokenResponse | null>(null);
const loading = ref(true);
const error = ref(false);
const copyConfirm = ref(false);
const saving = ref(false);

async function loadData(): Promise<void> {
  loading.value = true;
  error.value = false;
  try {
    const [userRes, token] = await Promise.all([
      fetch(`${import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080"}/me`, {
        headers: { Authorization: `Bearer ${(await import("../../auth/session")).getAccessToken()}` },
      }),
      getToken(),
    ]);
    if (userRes.ok) {
      userInfo.value = await userRes.json();
    }
    tokenInfo.value = token;
  } catch {
    error.value = true;
  } finally {
    loading.value = false;
  }
}

async function handleGenerate(): Promise<void> {
  saving.value = true;
  error.value = false;
  try {
    const response = await createToken();
    createdToken.value = response;
    tokenInfo.value = {
      id: response.id,
      token_prefix: response.token_prefix,
      created_at: response.created_at,
      last_used_at: null,
    };
  } catch {
    error.value = true;
  } finally {
    saving.value = false;
  }
}

async function handleRevoke(): Promise<void> {
  if (!confirm(t("labels.tokenRevokeConfirm"))) {
    return;
  }
  saving.value = true;
  error.value = false;
  try {
    await revokeToken();
    tokenInfo.value = null;
    createdToken.value = null;
  } catch {
    error.value = true;
  } finally {
    saving.value = false;
  }
}

async function handleCopyToken(): Promise<void> {
  if (!createdToken.value) return;
  try {
    await navigator.clipboard.writeText(createdToken.value.token_full);
    copyConfirm.value = true;
    setTimeout(() => { copyConfirm.value = false; }, 2000);
  } catch {
    error.value = true;
  }
}

onMounted(loadData);
</script>

<template>
  <main class="page-wrap">
    <h1 class="page-title">{{ t("labels.settingsPageTitle") }}</h1>

    <p v-if="error" class="error-message">{{ t("labels.tokenError") }}</p>

    <section v-if="userInfo" class="settings-section">
      <h2>{{ t("labels.settingsEmail") }}</h2>
      <p class="email-display">{{ userInfo.email }}</p>
    </section>

    <section class="settings-section">
      <h2>{{ t("labels.apiToken") }}</h2>

      <div v-if="loading" class="empty-state">...</div>

      <div v-else-if="createdToken" class="token-card token-generated">
        <p class="token-full-label">{{ t("labels.tokenGeneratedNotice") }}</p>
        <code class="token-full-display">{{ createdToken.token_full }}</code>
        <button class="icon-button" type="button" :title="t('labels.copyToken')" @click="handleCopyToken">
          <Check v-if="copyConfirm" :size="16" />
          <Copy v-else :size="16" />
          <span>{{ copyConfirm ? t("labels.tokenCopied") : t("labels.copyToken") }}</span>
        </button>
      </div>

      <div v-else-if="tokenInfo" class="token-card">
        <div class="token-detail">
          <span class="detail-label">{{ t("labels.tokenPrefix") }}</span>
          <span class="detail-value">{{ tokenInfo.token_prefix }}...</span>
        </div>
        <div class="token-detail">
          <span class="detail-label">{{ t("labels.tokenCreated") }}</span>
          <span class="detail-value">{{ new Date(tokenInfo.created_at).toLocaleString() }}</span>
        </div>
        <div class="token-detail">
          <span class="detail-label">{{ t("labels.tokenLastUsed") }}</span>
          <span class="detail-value">{{ tokenInfo.last_used_at ? new Date(tokenInfo.last_used_at).toLocaleString() : t("labels.never") }}</span>
        </div>
        <div class="token-actions">
          <button class="secondary-button" type="button" :disabled="saving" @click="handleGenerate">
            <RefreshCcw :size="14" />
            {{ t("labels.renewToken") }}
          </button>
          <button class="danger-button" type="button" :disabled="saving" @click="handleRevoke">
            <Trash2 :size="14" />
            {{ t("labels.revokeToken") }}
          </button>
        </div>
      </div>

      <div v-else class="token-card token-empty">
        <p>{{ t("labels.noApiToken") }}</p>
        <button class="primary-button" type="button" :disabled="saving" @click="handleGenerate">
          {{ t("labels.generateToken") }}
        </button>
      </div>
    </section>
  </main>
</template>

<style scoped>
.settings-section {
  margin-top: 1.5rem;
}

.settings-section h2 {
  font-size: 0.85rem;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: var(--muted-fg);
  margin: 0 0 0.6rem;
}

.email-display {
  font-size: 1.1rem;
  font-weight: 600;
}

.token-card {
  border: 1px solid var(--border-color);
  border-radius: 0.9rem;
  padding: 1rem;
  background: var(--panel-bg);
  display: grid;
  gap: 0.75rem;
}

.token-empty {
  align-items: start;
  gap: 0.8rem;
}

.token-empty p {
  margin: 0;
  color: var(--muted-fg);
}

.token-generated {
  gap: 0.6rem;
}

.token-full-label {
  font-size: 0.82rem;
  color: var(--muted-fg);
  margin: 0;
}

.token-full-display {
  display: block;
  padding: 0.7rem;
  background: rgba(0, 0, 0, 0.04);
  border-radius: 0.5rem;
  font-family: monospace;
  font-size: 0.82rem;
  word-break: break-all;
  line-height: 1.5;
}

html[data-theme="dark"] .token-full-display,
html[data-theme="system"][data-resolved-theme="dark"] .token-full-display {
  background: rgba(255, 255, 255, 0.05);
}

.token-detail {
  display: flex;
  gap: 0.5rem;
  align-items: baseline;
}

.detail-label {
  font-size: 0.82rem;
  color: var(--muted-fg);
  min-width: 80px;
}

.detail-value {
  font-weight: 600;
}

.token-actions {
  display: flex;
  gap: 0.6rem;
  margin-top: 0.3rem;
}

.icon-button {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  border: 1px solid var(--control-border);
  border-radius: 0.5rem;
  padding: 0.45rem 0.7rem;
  background: var(--control-bg);
  color: var(--control-fg);
  cursor: pointer;
  font-size: 0.85rem;
}

.secondary-button,
.primary-button,
.danger-button {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  border: 1px solid var(--control-border);
  border-radius: 0.55rem;
  padding: 0.45rem 0.7rem;
  cursor: pointer;
  font-size: 0.85rem;
}

.primary-button {
  border-color: rgba(72, 144, 255, 0.7);
  background: rgba(72, 144, 255, 0.22);
  color: var(--control-fg);
}

.danger-button {
  border-color: rgba(215, 58, 73, 0.5);
  color: #d73a49;
  background: transparent;
}

button:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.error-message {
  color: #f35a67;
  font-weight: 700;
}

.empty-state {
  color: var(--muted-fg);
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add front/src/views/settings/SettingsPage.vue
git commit -m "feat: add SettingsPage with token management UI"
```

---

### Task 12: Frontend tests

**Files:**
- Create: `front/src/views/settings/SettingsPage.test.ts`

- [ ] **Step 1: Create SettingsPage test**

Create `front/src/views/settings/SettingsPage.test.ts`:
```typescript
import { fireEvent, render, screen, waitFor } from "@testing-library/vue";
import { describe, expect, it, vi } from "vitest";
import { createAppI18n } from "../../common/i18n";
import SettingsPage from "./SettingsPage.vue";

vi.mock("./api", () => ({
  getToken: vi.fn(),
  createToken: vi.fn(),
  revokeToken: vi.fn(),
}));

import * as api from "./api";

function mockMe(email: string) {
  globalThis.fetch = vi.fn().mockResolvedValue({
    ok: true,
    json: async () => ({ email }),
  });
}

describe("Settings page", () => {
  it("shows email and generate button when no token exists", async () => {
    mockMe("test@example.com");
    vi.mocked(api.getToken).mockResolvedValue(null);

    render(SettingsPage, { global: { plugins: [createAppI18n("en")] } });

    expect(await screen.findByText("test@example.com")).not.toBeNull();
    expect(await screen.findByText("Generate token")).not.toBeNull();
  });

  it("shows token details when token exists", async () => {
    mockMe("test@example.com");
    vi.mocked(api.getToken).mockResolvedValue({
      id: "abc-123",
      token_prefix: "lc_abc",
      created_at: "2026-06-12T10:00:00Z",
      last_used_at: "2026-06-12T11:00:00Z",
    });

    render(SettingsPage, { global: { plugins: [createAppI18n("en")] } });

    expect(await screen.findByText("lc_abc...")).not.toBeNull();
    expect(screen.getByText("Renew token")).not.toBeNull();
    expect(screen.getByText("Revoke token")).not.toBeNull();
  });

  it("shows full token after generation", async () => {
    mockMe("test@example.com");
    vi.mocked(api.getToken).mockResolvedValue(null);
    vi.mocked(api.createToken).mockResolvedValue({
      id: "new-id",
      token_prefix: "lc_xyz",
      token_full: "lc_xyzabc123def456ghi789jkl012mno345pqr678stu901vwx",
      created_at: "2026-06-12T12:00:00Z",
    });

    render(SettingsPage, { global: { plugins: [createAppI18n("en")] } });

    const generateBtn = await screen.findByText("Generate token");
    await fireEvent.click(generateBtn);

    await waitFor(() => {
      expect(screen.getByText("This token won't be shown again.")).not.toBeNull();
    });
    expect(screen.getByText(/lc_xyzabc123def456ghi789jkl012mno345pqr678stu901vwx/)).not.toBeNull();
  });
});
```

- [ ] **Step 2: Run frontend tests**

Run: `npm run test` from `front/` (or `npx vitest run`)
Expected: all tests pass

- [ ] **Step 3: Commit**

```bash
git add front/src/views/settings/SettingsPage.test.ts
git commit -m "test: add SettingsPage tests"
```

---

### Task 13: Run full CI checks

- [ ] **Step 1: Backend checks**

Run: `cargo clippy --all-features -- -D warnings` from `back/`
Expected: clean, no warnings

Run: `cargo test --all-features` from `back/`
Expected: all tests pass

Run: `cargo build --all-features` from `back/`
Expected: builds successfully

- [ ] **Step 2: Frontend checks**

Run: `npm run lint` from `front/`
Expected: clean, no errors

Run: `npm run test` from `front/`
Expected: all tests pass

Run: `npm run build` from `front/`
Expected: builds successfully

---

### Self-Review Checklist

**Spec coverage:**
- DB migration: Task 1 ✓
- Backend model/repository: Task 2 ✓
- Backend service (generate, hash, verify, create, get, revoke): Task 3 ✓
- Backend HTTP handlers: Task 4 ✓
- Route registration: Task 5 ✓
- Principal extractor branching for `lc_` tokens: Task 6 ✓
- Backend tests: Task 7 ✓
- Frontend route + nav: Task 8 ✓
- i18n translations: Task 9 ✓
- Frontend API layer: Task 10 ✓
- Settings page UI (generate/copy/renew/revoke): Task 11 ✓
- Frontend tests: Task 12 ✓
- CI checks: Task 13 ✓

**Placeholder scan:** No TBDs, TODOs, or vague requirements in any code or test step.

**Type consistency:**
- `ApiToken` fields match DB columns ✓
- `CreateTokenResponse`/`TokenInfoResponse` match HTTP response shapes ✓
- Frontend types match backend types ✓
- Repository function signatures match service calls ✓
