# Design: Persistent Auth Across Browser Tabs

## Problem

Browser authentication is lost when opening a link in a new tab or restoring a closed tab. The user is redirected to the login page, creating a poor UX.

## Root Cause

1. **Refresh token stored in `sessionStorage`** (`front/src/auth/session.ts:43`) — `sessionStorage` is per-tab. Opening a new tab or restoring a closed tab creates a fresh `sessionStorage` with no refresh token.

2. **Access token stored in-memory only** (`session.ts:37-38`) — Lost on any page reload.

3. **Race condition on boot** — Both `router.beforeEach` and `AuthGuard.vue` call `ensureAuthenticated()` on page load. If both trigger `refreshAccessToken()` simultaneously with the same old token, the backend's token reuse detection may mark the session as compromised.

## Solution

### 1. `sessionStorage` → `localStorage` (`session.ts`)

Change `readRefreshToken`, `writeRefreshToken`, `clearRefreshToken` to use `localStorage`. `localStorage` is shared across all tabs for the same origin, so the refresh token survives new tabs and restored sessions.

Security note: `localStorage` has the same XSS surface as `sessionStorage` (both are readable by any JS on the origin). This is not a regression.

### 2. Dedup `refreshAccessToken()` (`session.ts`)

Add an in-flight promise guard. If a refresh is already in progress, concurrent callers await the same promise. This prevents the race condition where two simultaneous calls with the same old token trigger backend compromise detection.

### 3. Add `initAuth()` boot initializer (`session.ts`)

A function that eagerly starts token restoration. Called early in `App.vue`'s `<script setup>`, before the router guard fires. The cached promise is reused by subsequent callers.

### 4. Call `initAuth()` in `App.vue`

At the top of `<script setup>`, outside `onMounted`. This ensures the refresh starts as early as possible, before any component renders or navigation guards fire.

## Files Changed

- `front/src/auth/session.ts` — localStorage, dedup, initAuth
- `front/src/App.vue` — call initAuth
