# Authenticated Fetch Wrapper Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate "stale page" and "disconnection on i18n change" bugs by adding automatic token refresh before API calls and on 401 responses, with a hard limit of one retry.

**Architecture:** Create a single `authenticatedFetch` wrapper in the auth module that handles token lifecycle (proactive refresh, 401 recovery with exactly one retry, redirect to `/login` on failure). Modify all 6 existing API modules to use it instead of their duplicated `authHeaders` + `fetch` patterns. Simplify `AuthGuard.vue` to remove its redundant `ensureAuthenticated()` call.

**Tech Stack:** Vue 3, TypeScript, Vitest, native `fetch()` (no axios)

---

### Task 1: Create `authenticatedFetch` wrapper

**Files:**
- Create: `front/src/auth/authenticatedFetch.ts`
- Modify: `front/src/auth/session.ts` — export `clearSession` (already exported) and `getAccessToken`, no changes needed
- Test: `front/src/auth/authenticatedFetch.test.ts`

**Design rules:**
- Token refresh is attempted **before** the first call if `isAccessTokenUsable()` returns false
- On 401 response, exactly one refresh + retry is attempted
- If `refreshAccessToken()` returns false, call `window.location.href = "/login#/login"` (hard redirect, safe outside Vue context) and throw
- Uses the router only if available (injected via parameter or import); falls back to `window.location.href`
- Signature: `authenticatedFetch(url: string, options?: RequestInit): Promise<Response>`
- Does NOT parse JSON or throw on non-ok status — returns raw `Response` so callers keep their existing error handling

- [ ] **Step 1: Write the failing test for authenticatedFetch (Happy Path — token usable)**

```ts
import { describe, expect, it, vi, beforeEach } from "vitest";
import * as session from "../session";

// Mock the session module
vi.mock("../session", () => ({
  getAccessToken: vi.fn(),
  isAccessTokenUsable: vi.fn(),
  ensureAuthenticated: vi.fn(),
  refreshAccessToken: vi.fn(),
}));

describe("authenticatedFetch", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("should call fetch with auth header when token is usable", async () => {
    vi.mocked(session.isAccessTokenUsable).mockReturnValue(true);
    vi.mocked(session.getAccessToken).mockReturnValue("valid-token");

    const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValue(new Response("ok", { status: 200 }));

    const { authenticatedFetch } = await import("./authenticatedFetch");
    const response = await authenticatedFetch("https://api.example.com/data");

    expect(fetchSpy).toHaveBeenCalledWith("https://api.example.com/data", {
      headers: { Authorization: "Bearer valid-token" },
    });
    expect(response.status).toBe(200);
    fetchSpy.mockRestore();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npx vitest run front/src/auth/authenticatedFetch.test.ts --reporter=verbose`
Expected: FAIL — "Cannot find module"

- [ ] **Step 3: Write the basic authenticatedFetch implementation**

```ts
import { getAccessToken, isAccessTokenUsable, ensureAuthenticated, refreshAccessToken } from "./session";

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
    const refreshed = await refreshAccessToken();
    if (!refreshed) {
      redirectToLogin();
      throw new Error("session expired");
    }

    const retryHeaders = new Headers(options?.headers);
    retryHeaders.set("Authorization", `Bearer ${getAccessToken()}`);
    return fetch(url, {
      ...options,
      headers: retryHeaders,
    });
  }

  return response;
}

function redirectToLogin(): void {
  const currentPath = window.location.hash.replace(/^#/, "") || "/events";
  window.location.hash = `#/login?redirect=${encodeURIComponent(currentPath)}`;
}
```

- [ ] **Step 4: Write the 401 retry test**

```ts
it("should retry once on 401 if refresh succeeds", async () => {
  vi.mocked(session.isAccessTokenUsable).mockReturnValue(true);
  vi.mocked(session.getAccessToken)
    .mockReturnValueOnce("expired-token")
    .mockReturnValueOnce("new-token");
  vi.mocked(session.refreshAccessToken).mockResolvedValue(true);

  let callCount = 0;
  const fetchSpy = vi.spyOn(globalThis, "fetch").mockImplementation(async () => {
    callCount++;
    return new Response("ok", { status: callCount === 1 ? 401 : 200 });
  });

  const { authenticatedFetch } = await import("./authenticatedFetch");
  const response = await authenticatedFetch("https://api.example.com/data");

  expect(session.refreshAccessToken).toHaveBeenCalledTimes(1);
  expect(fetchSpy).toHaveBeenCalledTimes(2);
  expect(response.status).toBe(200);
  fetchSpy.mockRestore();
});
```

- [ ] **Step 5: Write the redirect-on-failure test**

```ts
it("should redirect to /login when refresh fails", async () => {
  vi.mocked(session.isAccessTokenUsable).mockReturnValue(true);
  vi.mocked(session.getAccessToken).mockReturnValue("expired-token");
  vi.mocked(session.refreshAccessToken).mockResolvedValue(false);

  const originalHash = window.location.hash;
  window.location.hash = "#/events/some-page";

  vi.spyOn(globalThis, "fetch").mockResolvedValue(new Response("unauthorized", { status: 401 }));

  const { authenticatedFetch } = await import("./authenticatedFetch");

  await expect(authenticatedFetch("https://api.example.com/data")).rejects.toThrow("session expired");
  expect(window.location.hash).toBe("#/login?redirect=%2Fevents%2Fsome-page");

  window.location.hash = originalHash;
});
```

- [ ] **Step 6: Run all tests to verify they pass**

Run: `npx vitest run front/src/auth/authenticatedFetch.test.ts --reporter=verbose`
Expected: PASS (all tests)

---

### Task 2: Migrate events, incidents, projects repositories to use `authenticatedFetch`

**Files:**
- Modify: `front/src/events/repositories/apiEventsRepository.ts`
- Modify: `front/src/incidents/repositories/apiIncidentsRepository.ts`
- Modify: `front/src/projects/repositories/apiProjectsRepository.ts`

**Pattern for each:** Replace the module-level `authHeaders()`, `fetchJson()`, `fetchJsonOrNull()`, `sendJson()` functions with equivalents that call `authenticatedFetch` instead of raw `fetch`. The class methods stay unchanged — only the helpers change.

- [ ] **Step 7: Add authenticatedFetch import and update helpers in apiEventsRepository.ts**

Replace:
```ts
import { getAccessToken } from "../../auth/session";

function authHeaders(): HeadersInit {
  const token = getAccessToken();
  if (!token) {
    throw new Error("missing access token");
  }
  return { Authorization: `Bearer ${token}` };
}

async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(url, { headers: authHeaders() });
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

async function fetchJsonOrNull<T>(url: string): Promise<T | null> {
  const response = await fetch(url, { headers: authHeaders() });
  if (response.status === 404) return null;
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

async function sendJson<T>(url: string, method: string, body?: unknown): Promise<T | null> {
  const response = await fetch(url, {
    method,
    headers: { ...authHeaders(), "Content-Type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return response.status === 204 ? null : ((await response.json()) as T);
}
```

With:
```ts
import { authenticatedFetch } from "../../auth/authenticatedFetch";

async function fetchJson<T>(url: string): Promise<T> {
  const response = await authenticatedFetch(url);
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

async function fetchJsonOrNull<T>(url: string): Promise<T | null> {
  const response = await authenticatedFetch(url);
  if (response.status === 404) return null;
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

async function sendJson<T>(url: string, method: string, body?: unknown): Promise<T | null> {
  const response = await authenticatedFetch(url, {
    method,
    headers: { "Content-Type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return response.status === 204 ? null : ((await response.json()) as T);
}
```

Also remove the unused `getAccessToken` import.

- [ ] **Step 8: Repeat the same replacement in apiIncidentsRepository.ts** (identical pattern, same line ranges ~160-199)

Replace the module-level `authHeaders()`, `fetchJson()`, `fetchJsonOrNull()`, `sendJson()` functions identically to Step 7.

- [ ] **Step 9: Repeat the same replacement in apiProjectsRepository.ts** (identical pattern, same line ranges ~139-176)

Replace the module-level `authHeaders()`, `fetchJson()`, `fetchJsonOrNull()`, `sendJson()` functions identically to Step 7.

Note: `apiProjectsRepository.ts` has a minor difference in `authHeaders()` — it checks `import.meta.env.MODE !== "test"` and conditionally throws. With `authenticatedFetch`, this test-mode bypass is no longer needed because test mode uses mock repos (see `mockProjectsRepository`). The `useMockData()` check in each class constructor already handles this.

- [ ] **Step 10: Run existing tests to verify no regressions**

Run: `npx vitest run --reporter=verbose`
Expected: PASS (all existing tests including `apiProjectsRepository.test.ts`)

---

### Task 3: Migrate categories, admin, settings API modules to use `authenticatedFetch`

**Files:**
- Modify: `front/src/categories/api.ts`
- Modify: `front/src/admin/api.ts`
- Modify: `front/src/views/settings/api.ts`

These standalone modules follow a simpler pattern — each has `authHeaders()` and a single `request()`/`fetchJson()` helper.

- [ ] **Step 11: Migrate categories/api.ts**

Replace:
```ts
import { getAccessToken } from "../auth/session";

function authHeaders(json = false): HeadersInit {
  const token = getAccessToken();
  if (!token && import.meta.env.MODE !== "test") {
    throw new Error("missing access token");
  }
  return {
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
    ...(json ? { "Content-Type": "application/json" } : {}),
  };
}

async function request<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await fetch(url, init);
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}
```

With:
```ts
import { authenticatedFetch } from "../auth/authenticatedFetch";

async function request<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await authenticatedFetch(url, init);
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}
```

Update callers: `listCategories()` calls `request(url, { headers: authHeaders() })` → change to `request(url, { headers: { "Content-Type": "application/json" } })` for the admin variant (auth is handled by `authenticatedFetch`). Actually, `listCategories` passes `{ headers: authHeaders() }` without JSON content type. Since `authenticatedFetch` adds the `Authorization` header automatically, the callers should not pass auth headers. Update each function call accordingly:

- `listCategories` — replace `request<CategoryItem[]>(url, { headers: authHeaders() })` with `request<CategoryItem[]>(url)`
- `createCategory` — replace `fetch(url, { method: "POST", headers: authHeaders(true), body: ... })` with `authenticatedFetch(url, { method: "POST", headers: { "Content-Type": "application/json" }, body: ... })` and add its own error handling
- `updateCategory` — same as createCategory
- `deleteCategory` — replace `fetch(url, { method: "DELETE", headers: authHeaders() })` with `authenticatedFetch(url, { method: "DELETE" })` with error handling

For the non-request helpers (`createCategory`, `updateCategory`, `deleteCategory`), use `authenticatedFetch` directly and duplicate the error handling pattern from `request()`. This is YAGNI — creating a `sendJson` equivalent for one module would be over-engineering.

- [ ] **Step 12: Migrate admin/api.ts**

Replace `authHeaders()` and `fetchJson()` with `authenticatedFetch` import and equivalent helper:

```ts
import { authenticatedFetch } from "../auth/authenticatedFetch";

async function fetchJson<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await authenticatedFetch(url, init);
  if (!response.ok) {
    throw new Error(`request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}
```

Remove `authHeaders()` entirely. Update all callers:
- `listRoles` — `{ headers: authHeaders() }` → remove the empty init (pass nothing), authenticatedFetch adds auth
- `listUsers` — same
- `updateUserRoles` — `{ headers: authHeaders(true), body: ... }` → `{ headers: { "Content-Type": "application/json" }, body: ... }`
- `updateUserNames` — same

- [ ] **Step 13: Migrate settings/api.ts**

Same pattern as admin/api.ts: remove `authHeaders()`, replace `fetchJson()` helper to use `authenticatedFetch`, update callers.

- [ ] **Step 14: Run all tests to verify no regressions**

Run: `npx vitest run --reporter=verbose`
Expected: PASS

---

### Task 4: Remove redundant auth check from AuthGuard.vue

**Files:**
- Modify: `front/src/auth/AuthGuard.vue`

**Why safe:** `router.beforeEach` (router/index.ts:52-95) already calls `ensureAuthenticated()` before the route component mounts. AuthGuard's duplicate check adds no value and creates a second async call on every route change.

- [ ] **Step 15: Simplify AuthGuard.vue**

Remove the entire `<script setup>` content and replace with a passthrough slot:

```vue
<template>
  <slot />
</template>
```

- [ ] **Step 16: Verify the app still compiles**

Run: `npm run build`
Expected: SUCCESS (no TypeScript errors, bundle builds)

---

### Task 5: E2E verification

- [ ] **Step 17: Run full frontend CI checks**

Run: `npm run lint` and `npm run test` and `npm run build`
Expected: PASS all three

---

## Self-Review

1. **Spec coverage:**
   - Issue 1 (stale page): ✓ `authenticatedFetch` proactively refreshes token before calls and retries once on 401
   - Issue 2 (i18n disconnection): ✓ Same fix — locale-triggered refetches go through `authenticatedFetch`
   - Redirect on failed refresh: ✓ `redirectToLogin()` is called immediately when `refreshAccessToken()` returns false
   - No infinite loop: ✓ Exactly one retry; second 401 propagates as error

2. **Placeholder scan:** No TODOs, TBDs, or "implement later" patterns found.

3. **Type consistency:** All callers updated to use the same `authenticatedFetch(url, init?)` signature. No type drift between tasks.

4. **Edge cases covered:**
   - Token expired before call → proactive refresh → call succeeds
   - Token valid → call returns 401 (race condition) → refresh → retry → success
   - Refresh fails → redirect to /login → no retry
   - Refresh succeeds but retry also 401 (server revoked session) → error propagates
   - Concurrent API calls all hitting 401 → `refreshAccessToken()` singleton prevents multiple refreshes
