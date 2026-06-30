# Mobile Performance Fixes — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce mobile page-load time and network chattiness by fixing 5 specific performance problems identified during codebase analysis.

**Architecture:** 5 independent fixes — 4 frontend (bundle splitting, `/me` caching, initAuth de-risking, 401 retry removal), 1 backend (DB query reduction in `/refresh`). Each is a single-file change affecting one concern.

**Tech Stack:** Vue 3 + Vite (front), Rust/Axum/SQLx (back)

---

### Task 1: Manual chunk splitting for @scalar/api-reference

**Files:**
- Modify: `front/vite.config.ts`

**Problem:** `@scalar/api-reference` (used only by lazy-loaded `ApiDocPage.vue`) is bundled into a shared `bus-*.js` chunk of ~2.1 MB. Vite's default code splitting groups it with other shared imports. This chunk is loaded on every page access even when the user never visits the API docs page.

**Fix:** Add `build.rollupOptions.output.manualChunks` to split `@scalar/api-reference` into its own async chunk (`scalar.js`). Because the component is already lazy-imported (`() => import()`), Vite will only load this chunk when the user navigates to `/settings/api-doc`.

- [ ] **Step 1: Edit `vite.config.ts`**

    Replace the current content with:

    ```ts
    import { defineConfig } from "vite";
    import vue from "@vitejs/plugin-vue";
    import { configDefaults } from "vitest/config";

    export default defineConfig({
      plugins: [vue()],
      build: {
        rollupOptions: {
          output: {
            manualChunks: {
              scalar: ["@scalar/api-reference"],
            },
          },
        },
      },
      test: {
        environment: "jsdom",
        setupFiles: [],
        globals: true,
        exclude: [...configDefaults.exclude, "tests/e2e/**"],
      },
    });
    ```

- [ ] **Step 2: Verify build**

    Run: `cd front; npm run build`

    Expected: Build succeeds. The output should show a separate chunk like `scalar-xxxxx.js` instead of `bus-*.js` containing scalar code.

- [ ] **Step 3: Verify tests still pass**

    Run: `cd front; npm run test`

    Expected: All tests pass (no logic changed, only build config).

- [ ] **Step 4: Commit**

    ```bash
    git add front/vite.config.ts
    git commit -m "perf: split @scalar/api-reference into async chunk"
    ```

---

### Task 2: Cache /me responses between navigations

**Files:**
- Modify: `front/src/auth/session.ts`

**Problem:** Router guard calls `ensureCurrentUserRoles()` on every SPA navigation. This always reaches `fetchCurrentUserRoles()` which makes a blocking `GET /me` API call with 2 DB queries. User roles don't change mid-session.

**Fix:** In `ensureCurrentUserRoles()`, return immediately if `currentUserRoles.loaded` is already true. The `loaded` flag is already set to `false` in `clearSession()` and to `true` after the first successful fetch, so this correctly handles session expiry and page reload.

- [ ] **Step 1: Edit `session.ts` — add early return in `ensureCurrentUserRoles()`**

    In `front/src/auth/session.ts`, find the `ensureCurrentUserRoles` function at line 241. Add an early return after the `useMockData()` block, before the `currentUserRolesRequest` check:

    Old:
    ```ts
    export async function ensureCurrentUserRoles(): Promise<boolean> {
      if (useMockData()) {
        currentUserId.value = "00000000-0000-0000-0000-000000000000";
        currentUserRoles.roles = ["ADMIN", "CO_OWNER", "CO_OWNERSHIP_BOARD"];
        currentUserRoles.loaded = true;
        return true;
      }

      if (currentUserRolesRequest) {
        return currentUserRolesRequest;
      }
    ```

    New:
    ```ts
    export async function ensureCurrentUserRoles(): Promise<boolean> {
      if (useMockData()) {
        currentUserId.value = "00000000-0000-0000-0000-000000000000";
        currentUserRoles.roles = ["ADMIN", "CO_OWNER", "CO_OWNERSHIP_BOARD"];
        currentUserRoles.loaded = true;
        return true;
      }

      if (currentUserRoles.loaded) {
        return true;
      }

      if (currentUserRolesRequest) {
        return currentUserRolesRequest;
      }
    ```

- [ ] **Step 2: Verify tests**

    Run: `cd front; npm run test`

    Expected: All tests pass. The existing test suite covers `ensureAuthenticated` but `ensureCurrentUserRoles` is tested indirectly through router guard tests.

- [ ] **Step 3: Run linter**

    Run: `cd front; npm run lint`

    Expected: No type errors.

- [ ] **Step 4: Commit**

    ```bash
    git add front/src/auth/session.ts
    git commit -m "perf: cache /me response to avoid fetch on each navigation"
    ```

---

### Task 3: Eliminate initOAuthSession() on every page load

**Files:**
- Modify: `front/src/auth/session.ts`
- Modify: `front/src/auth/LoginPage.vue`

**Problem:** `initAuth()` in `App.vue:8` calls `initOAuthSession()` on every full page load. This fires a `GET /auth/session` request that creates/refreshes an OAuth session cookie. But the OAuth session is only needed when the user clicks "Sign in with Google/Microsoft" — the `providerStartUrl()` function already triggers cookie creation via the OAuth `start` endpoint.

The `/auth/session` endpoint itself (backend `init_session` at `service.rs:85`) just calls `get_or_create_session_id()` which creates a session_id cookie if none exists. This is unnecessary for already-authenticated users who already have a valid session.

**Fix:** Remove the `initOAuthSession()` call from `initAuth()`. Keep the exported function for use by the login flow.

- [ ] **Step 1: Edit `session.ts` — remove `initOAuthSession()` from `initAuth()`**

    In `front/src/auth/session.ts`, find `initAuth()` at line 173. Remove the `initOAuthSession();` call:

    Old:
    ```ts
    export function initAuth(): Promise<boolean> {
      if (authInitPromise) {
        return authInitPromise;
      }
      if (useMockData()) {
        authInitPromise = Promise.resolve(true);
        return authInitPromise;
      }
      initOAuthSession();
      authInitPromise = ensureAuthenticated();
      return authInitPromise;
    }
    ```

    New:
    ```ts
    export function initAuth(): Promise<boolean> {
      if (authInitPromise) {
        return authInitPromise;
      }
      if (useMockData()) {
        authInitPromise = Promise.resolve(true);
        return authInitPromise;
      }
      authInitPromise = ensureAuthenticated();
      return authInitPromise;
    }
    ```

- [ ] **Step 2: Edit `LoginPage.vue` — call `initOAuthSession()` on login page mount**

    In `front/src/auth/LoginPage.vue`, add the import and lifecycle hook to initialize the OAuth session before the user clicks a sign-in button:

    Old imports:
    ```ts
    import { computed } from "vue";
    import { useRoute } from "vue-router";
    import { sanitizeRedirectPath } from "./redirect";
    import { providerStartUrl } from "./session";
    ```

    New imports:
    ```ts
    import { computed, onMounted } from "vue";
    import { useRoute } from "vue-router";
    import { sanitizeRedirectPath } from "./redirect";
    import { initOAuthSession, providerStartUrl } from "./session";
    ```

    Add after the `const microsoftHref` line:
    ```ts
    onMounted(() => {
      initOAuthSession();
    });
    ```

- [ ] **Step 3: Verify tests**

    Run: `cd front; npm run test`

    Expected: All tests pass.

- [ ] **Step 4: Run linter**

    Run: `cd front; npm run lint`

    Expected: No type errors.

- [ ] **Step 5: Commit**

    ```bash
    git add front/src/auth/session.ts front/src/auth/LoginPage.vue
    git commit -m "perf: only init oauth session on login page, not every pageload"
    ```

---

### Task 4: Remove dead 401→refresh retry logic from authenticatedFetch

**Files:**
- Modify: `front/src/auth/authenticatedFetch.ts`
- Modify: `front/src/auth/authenticatedFetch.test.ts`

**Problem:** `doFetch()` has a 401 retry mechanism (`refreshAccessToken()` + retry with `retried` flag) that is dead code in practice. Before any `authenticatedFetch` call, the router guard already ensures a usable access token via `ensureAuthenticated()` in the `beforeEach` guard. Additionally, within `doFetch()` itself, `isAccessTokenUsable()` is checked and `ensureAuthenticated()` is called if needed. So by the time `fetch()` is called, the token should always be valid. A 401 at this point means the token was revoked or the session was terminated server-side — retrying with a refresh will produce the same result (the old session is gone).

**Fix:** Remove the `retried` parameter and the `if (response.status === 401 && !retried)` block. On 401, redirect to login immediately. This eliminates the `refreshAccessToken()` import and simplifies `doFetch()`.

- [ ] **Step 1: Edit `authenticatedFetch.ts` — simplify `doFetch()`**

    In `front/src/auth/authenticatedFetch.ts`, replace the entire file:

    Old:
    ```ts
    import { getAccessToken, isAccessTokenUsable, ensureAuthenticated, refreshAccessToken } from "./session";

    export async function authenticatedFetch(url: string, options?: RequestInit): Promise<Response> {
      return doFetch(url, options, false);
    }

    async function doFetch(url: string, options?: RequestInit, retried?: boolean): Promise<Response> {
      if (!isAccessTokenUsable()) {
        const ok = await ensureAuthenticated();
        if (!ok) {
          redirectToLogin();
          throw new Error("session expired");
        }
      }

      const headers = new Headers(options?.headers);
      headers.set("Authorization", `Bearer ${getAccessToken()}`);

      const response = await fetch(url, {
        ...options,
        headers,
      });

      if (response.status === 401 && !retried) {
        const refreshed = await refreshAccessToken();
        if (!refreshed) {
          redirectToLogin();
          throw new Error("session expired");
        }

        return doFetch(url, options, true);
      }

      return response;
    }

    export function currentRedirectPath(): string {
      return window.location.hash.replace(/^#/, "") || "/events";
    }

    function redirectToLogin(): void {
      window.location.hash = `#/login?redirect=${encodeURIComponent(currentRedirectPath())}`;
    }
    ```

    New:
    ```ts
    import { getAccessToken, isAccessTokenUsable, ensureAuthenticated } from "./session";

    export async function authenticatedFetch(url: string, options?: RequestInit): Promise<Response> {
      if (!isAccessTokenUsable()) {
        const ok = await ensureAuthenticated();
        if (!ok) {
          redirectToLogin();
          throw new Error("session expired");
        }
      }

      const headers = new Headers(options?.headers);
      headers.set("Authorization", `Bearer ${getAccessToken()}`);

      const response = await fetch(url, {
        ...options,
        headers,
      });

      if (response.status === 401) {
        redirectToLogin();
        throw new Error("session expired");
      }

      return response;
    }

    export function currentRedirectPath(): string {
      return window.location.hash.replace(/^#/, "") || "/events";
    }

    function redirectToLogin(): void {
      window.location.hash = `#/login?redirect=${encodeURIComponent(currentRedirectPath())}`;
    }
    ```

- [ ] **Step 2: Update test file — remove retry tests, adjust remaining**

    In `front/src/auth/authenticatedFetch.test.ts`, update imports and remove retry-specific tests:

    Old imports mock:
    ```ts
    import * as session from "./session";

    vi.mock("./session", () => ({
      getAccessToken: vi.fn(),
      isAccessTokenUsable: vi.fn(),
      ensureAuthenticated: vi.fn(),
      refreshAccessToken: vi.fn(),
    }));
    ```

    New imports mock:
    ```ts
    import * as session from "./session";

    vi.mock("./session", () => ({
      getAccessToken: vi.fn(),
      isAccessTokenUsable: vi.fn(),
      ensureAuthenticated: vi.fn(),
    }));
    ```

    Remove these test cases (lines 49-137 are the retry tests that should be deleted):
    - "retries once on 401 if refresh succeeds" (line 49)
    - "redirects to /login when refresh fails on 401" (line 71)
    - "preserves caller's Content-Type header" (line 104)
    - "does not retry more than once" (line 124)

    Replace them with a single new test for the simplified 401 behavior:

    ```ts
    it("redirects to login on 401 response", async () => {
      vi.mocked(session.isAccessTokenUsable).mockReturnValue(true);
      vi.mocked(session.getAccessToken).mockReturnValue("token");

      const originalHash = window.location.hash;
      window.location.hash = "#/events";

      vi.spyOn(globalThis, "fetch").mockResolvedValue(new Response("unauthorized", { status: 401 }));

      const { authenticatedFetch } = await import("./authenticatedFetch");

      await expect(authenticatedFetch("https://api.example.com/data")).rejects.toThrow("session expired");
      expect(window.location.hash).toBe("#/login?redirect=%2Fevents");

      window.location.hash = originalHash;
    });
    ```

    Keep these existing tests unchanged:
    - "calls fetch with auth header when token is usable" (line 16)
    - "proactively refreshes token before call if not usable" (line 32)

    Also remove the import of `refreshAccessToken` from the destructured session mock (it's already removed from the mock).

- [ ] **Step 3: Verify tests**

    Run: `cd front; npm run test`

    Expected: All tests pass. The remaining tests verify the core behavior (auth header injection, proactive refresh before call, and 401 redirect).

- [ ] **Step 4: Run linter**

    Run: `cd front; npm run lint`

    Expected: No type errors.

- [ ] **Step 5: Commit**

    ```bash
    git add front/src/auth/authenticatedFetch.ts front/src/auth/authenticatedFetch.test.ts
    git commit -m "perf: remove dead 401-retry logic from authenticatedFetch"
    ```

---

### Task 5: Optimize /refresh backend — combine DB queries

**Files:**
- Modify: `back/src/auth/repository.rs`
- Modify: `back/src/auth/model.rs`

**Problem:** The `/refresh` handler (`service.rs:refresh`) makes 3 DB queries:
1. `rotate_session_refresh_token` — UPDATE + RETURNING `(id, user_id)`
2. `get_user_by_id` — SELECT from users by PK
3. `mark_user_login` — UPDATE users

The `get_user_by_id` call is wasteful because the `rotate_session_refresh_token` query already knows the `user_id`. We can modify the UPDATE to JOIN with `users` and return user data directly, eliminating query #2.

**Fix:** Modify `RotatedSession` to include user fields, update the SQL to JOIN `users`, and update the service to use the returned data.

- [ ] **Step 1: Modify `RotatedSession` struct in `model.rs`**

    In `back/src/auth/model.rs`, find the `RotatedSession` definition (or add if it only exists in repository.rs). Actually, `RotatedSession` is defined in `repository.rs`. We'll modify it there.

    In `back/src/auth/repository.rs`, update the `RotatedSession` struct to include user fields:

    Old:
    ```rust
    pub struct RotatedSession {
        pub id: Uuid,
        pub user_id: Uuid,
    }
    ```

    New:
    ```rust
    pub struct RotatedSession {
        pub id: Uuid,
        pub user_id: Uuid,
        pub provider: String,
        pub email: String,
        pub first_name: Option<String>,
        pub last_name: Option<String>,
        pub roles: Vec<String>,
    }
    ```

- [ ] **Step 2: Update the SQL query in `rotate_session_refresh_token`**

    In `back/src/auth/repository.rs`, modify the `rotate_session_refresh_token` function to JOIN with `users` table:

    Old SQL:
    ```sql
    UPDATE auth_sessions
    SET previous_refresh_token_hashes = (
          CASE
            WHEN array_length(previous_refresh_token_hashes, 1) IS NULL
              THEN ARRAY[refresh_token_hash]
            WHEN array_length(previous_refresh_token_hashes, 1) >= 8
              THEN previous_refresh_token_hashes[2:8] || refresh_token_hash
            ELSE previous_refresh_token_hashes || refresh_token_hash
          END
        ),
        refresh_token_hash = $2,
        expires_at = $3,
        updated_at = now()
    WHERE refresh_token_hash = $1
      AND revoked_at IS NULL
      AND compromised_at IS NULL
      AND expires_at > now()
    RETURNING id, user_id
    ```

    New SQL:
    ```sql
    UPDATE auth_sessions s
    SET previous_refresh_token_hashes = (
          CASE
            WHEN array_length(previous_refresh_token_hashes, 1) IS NULL
              THEN ARRAY[refresh_token_hash]
            WHEN array_length(previous_refresh_token_hashes, 1) >= 8
              THEN previous_refresh_token_hashes[2:8] || refresh_token_hash
            ELSE previous_refresh_token_hashes || refresh_token_hash
          END
        ),
        refresh_token_hash = $2,
        expires_at = $3,
        updated_at = now()
    FROM users u
    WHERE s.refresh_token_hash = $1
      AND s.user_id = u.id
      AND s.revoked_at IS NULL
      AND s.compromised_at IS NULL
      AND s.expires_at > now()
    RETURNING s.id, s.user_id, u.provider, u.email, u.first_name, u.last_name, u.roles
    ```

    Update the `row.map` closure to extract the new fields:

    Old:
    ```rust
    row.map(|row| {
        Ok(RotatedSession {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
        })
    })
    .transpose()
    ```

    New:
    ```rust
    row.map(|row| {
        Ok(RotatedSession {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            provider: row.try_get("provider")?,
            email: row.try_get("email")?,
            first_name: row.try_get("first_name")?,
            last_name: row.try_get("last_name")?,
            roles: row.try_get("roles")?,
        })
    })
    .transpose()
    ```

- [ ] **Step 3: Update `/refresh` handler in `service.rs` to use user data from `RotatedSession`**

    In `back/src/auth/service.rs`, modify the `refresh` function to use the user data returned from `rotate_session_refresh_token` instead of calling `get_user_by_id`:

    Old (lines 305-331):
    ```rust
    let Some(rotated) = repository::rotate_session_refresh_token(
        &state.db,
        &incoming_hash,
        &hash_token(&next_refresh_token),
        Utc::now() + ChronoDuration::seconds(REFRESH_SESSION_TTL.as_secs() as i64),
    )
    .await?
    else {
        if let Some(reused) = repository::find_session_by_previous_refresh_hash(&state.db, &incoming_hash).await? {
            repository::compromise_session(&state.db, reused.id).await?;
            return Err(AppError::unauthorized("refresh token reuse detected"));
        }
        return Err(AppError::unauthorized("invalid refresh token"));
    };

    let user = repository::get_user_by_id(&state.db, rotated.user_id)
        .await?
        .ok_or_else(|| AppError::unauthorized("invalid user"))?;
    repository::mark_user_login(&state.db, rotated.user_id).await?;

    let (access_token, expires_at) = issue_access_token(state, rotated.id, &user)?;
    ```

    New:
    ```rust
    let Some(rotated) = repository::rotate_session_refresh_token(
        &state.db,
        &incoming_hash,
        &hash_token(&next_refresh_token),
        Utc::now() + ChronoDuration::seconds(REFRESH_SESSION_TTL.as_secs() as i64),
    )
    .await?
    else {
        if let Some(reused) = repository::find_session_by_previous_refresh_hash(&state.db, &incoming_hash).await? {
            repository::compromise_session(&state.db, reused.id).await?;
            return Err(AppError::unauthorized("refresh token reuse detected"));
        }
        return Err(AppError::unauthorized("invalid refresh token"));
    };

    repository::mark_user_login(&state.db, rotated.user_id).await?;

    let (access_token, expires_at) = issue_access_token(state, rotated.id, &rotated)?;
    ```

    Also update the `issue_access_token` call to accept `&RotatedSession` instead of `&DbUser`. The `issue_access_token` function expects `&DbUser` which has `id`, `provider`, `email`, `first_name`, `last_name`, `roles` fields. Our updated `RotatedSession` has matching fields with matching types, so the call will work.

    Actually, `issue_access_token` takes `&DbUser`. Let me check its signature:

    ```rust
    fn issue_access_token(
        state: &AppState,
        session_id: Uuid,
        user: &repository::DbUser,
    ) -> Result<(String, SystemTime), AppError> {
    ```

    The function expects `&DbUser`. Our `RotatedSession` now has the same fields but it's a different type. We have two options:
    a) Change `issue_access_token` to accept a common trait or the `RotatedSession` directly
    b) Keep both and create a `DbUser` from the `RotatedSession` data

    Since `issue_access_token` is also called from `exchange_session`, and that function uses `DbUser`, the cleanest approach is to accept either type. We can either:
    - Extract the common fields into an interface/trait
    - Change `issue_access_token` to take individual fields
    - Convert `RotatedSession` to `DbUser` temporarily

    The simplest approach: change `issue_access_token` to accept user data via individual fields or a slice of the needed fields. Actually the cleanest fix is to refactor `issue_access_token` to take the specific fields it needs:

    ```rust
    fn issue_access_token(
        state: &AppState,
        session_id: Uuid,
        user_id: &Uuid,
        user_provider: &str,
    ) -> Result<(String, SystemTime), AppError> {
    ```

    Wait, looking at the function body more carefully:

    ```rust
    fn issue_access_token(
        state: &AppState,
        session_id: Uuid,
        user: &repository::DbUser,
    ) -> Result<(String, SystemTime), AppError> {
        let now_unix = unix_ts(SystemTime::now())?;
        let exp_unix = now_unix + ACCESS_TOKEN_TTL.as_secs();
        let claims = AccessTokenClaims {
            sub: user.id.to_string(),
            sid: session_id.to_string(),
            provider: user.provider.clone(),
            email: user.email.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            roles: user.roles.clone(),
            iat: now_unix as usize,
            exp: exp_unix as usize,
        };
    ```

    It needs `user.id`, `user.provider`, `user.email`, `user.first_name`, `user.last_name`, `user.roles`. All of these are now available in `RotatedSession`.

    The cleanest approach: keep `DbUser` as the canonical type for the `me` and `exchange_session` paths, but make `issue_access_token` generic by accepting the common interface. We can use a simple approach — define a private trait or just take the needed fields.

    Actually, the simplest approach that maintains backward compatibility: implement `From<RotatedSession> for DbUser` or just change the `issue_access_token` to accept the individual fields it needs.

    Let me go with the simplest: change `issue_access_token` to accept a generic reference that provides the needed fields. Since Rust doesn't have duck typing, I'll create a tiny helper trait or just change the signature to take `&RotatedSession` and make the other caller (`exchange_session`) construct a `RotatedSession` from `DbUser`.

    Actually, the even simpler approach: just keep `issue_access_token` taking `&DbUser` and convert `RotatedSession` → `DbUser` inline:

    ```rust
    let user = repository::DbUser {
        id: rotated.user_id,
        provider: rotated.provider.clone(),
        email: rotated.email.clone(),
        first_name: rotated.first_name.clone(),
        last_name: rotated.last_name.clone(),
        roles: rotated.roles.clone(),
    };
    let (access_token, expires_at) = issue_access_token(state, rotated.id, &user)?;
    ```

    This is clean and doesn't change any existing function signatures. The user struct is stack-allocated and short-lived.

- [ ] **Step 4: Build and verify backend compiles**

    Run: `cd back; cargo build --all-features`

    Expected: Build succeeds with no warnings about unused imports (check `DbUser` import).

- [ ] **Step 5: Run backend tests**

    Run: `cd back; cargo test --all-features`

    Expected: All tests pass.

- [ ] **Step 6: Run clippy**

    Run: `cd back; cargo clippy --all-features -- -D warnings`

    Expected: No warnings.

- [ ] **Step 7: Commit**

    ```bash
    git add back/src/auth/repository.rs back/src/auth/service.rs
    git commit -m "perf: eliminate redundant DB query in /refresh by JOINing users"
    ```
