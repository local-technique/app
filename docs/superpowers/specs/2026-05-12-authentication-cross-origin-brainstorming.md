# Authentication Brainstorming (GitHub Pages Frontend + Cross-Origin Rust Backend)

## 1) Context and Constraints

- Frontend stays as a static SPA on GitHub Pages: `https://local-technique.github.io/app/`
- Backend will be a Rust API hosted on Leapcell (different origin/domain)
- Target authentication providers: Google and Facebook OAuth
- Audience is broad and mostly mobile-first; low-friction sign-in is important for adoption
- Project is associative (not corporate), but trust and basic security are still important

## 2) Core Browser Reality

Cross-origin API calls are allowed by browsers when backend CORS policy is correct.

- Browser sends `Origin: https://local-technique.github.io`
- Backend must explicitly allow that origin
- Backend must allow required headers (notably `Authorization`)

This means the architecture is feasible without a custom domain.

## 3) Auth Architecture Options

### Option A: Cross-site cookie session (BFF style)

- Backend sets `HttpOnly; Secure; SameSite=None` auth cookies
- Frontend sends `withCredentials` requests

Pros:
- Tokens are hidden from JavaScript (`HttpOnly`)

Cons:
- Cross-site/third-party cookie restrictions are increasingly strict and inconsistent, especially on mobile
- Requires CSRF protections for state-changing endpoints
- More browser-specific edge cases

### Option B: SPA bearer tokens (recommended)

- Frontend sends `Authorization: Bearer <access_token>`
- Backend issues app JWTs and refresh tokens after OAuth login

Pros:
- No dependency on third-party cookies
- Predictable with CORS
- Straightforward axios/fetch flow

Cons:
- Any token readable by JS is exposed if XSS occurs

### Option C: Same-origin proxy façade

- Put frontend and API behind one origin via reverse proxy/custom domain

Pros:
- Best compatibility and cookie-based security model

Cons:
- Conflicts with current no-domain/no-infra constraint

## 4) Recommended Flow

Use OAuth Authorization Code + PKCE with backend-managed provider exchange.

1. User taps "Continue with Google" or "Continue with Facebook"
2. Frontend redirects to backend `/auth/{provider}/start`
3. Backend generates `state` (+ PKCE), redirects to provider
4. Provider redirects to backend callback
5. Backend validates `state`, exchanges code, identifies user
6. Backend issues:
   - short-lived access JWT
   - long-lived rotating refresh token
7. Frontend uses access JWT in `Authorization` header
8. Frontend silently refreshes when needed

Provider client secrets remain backend-only.

## 5) Storage Strategy and UX Tradeoff

### Key risk reminder

`localStorage` and `sessionStorage` are origin-scoped, but XSS on this origin can still:

- Read and exfiltrate tokens stored there
- Call the API as the current user

So impact is mostly account takeover inside this app (not arbitrary takeover of unrelated domains).

### Practical storage choices

- `In-memory` access token: safer, but lost on reload/new tab
- `sessionStorage`: survives reload in tab, not across tabs/sessions
- `localStorage`: survives browser restarts, best UX, higher XSS exposure

### Recommended compromise for this project

- Store access token in memory only (short TTL)
- Store refresh token in `localStorage` (rotating, revocable)
- On app boot, call `/auth/refresh` silently to restore session

This minimizes frequent re-authentication while keeping short-lived access tokens.

## 6) Token Lifetime Policy (Production-Lite)

Balanced default for mobile UX and acceptable risk:

- Access token TTL: `10-15 minutes`
- Refresh token: `30 days` sliding window
- Absolute refresh lifetime cap: `90 days`
- Rotate refresh token on every refresh
- Detect refresh token reuse and revoke token family on replay

Expected UX:

- Most users stay signed in for weeks
- Provider re-login (Google/Facebook) happens mainly on explicit logout, long inactivity, revocation, or security events

## 7) CORS Policy Baseline

Backend should return:

- `Access-Control-Allow-Origin: https://local-technique.github.io`
- `Vary: Origin`
- `Access-Control-Allow-Methods: GET, POST, PUT, PATCH, DELETE, OPTIONS`
- `Access-Control-Allow-Headers: Authorization, Content-Type`

For bearer-token model:

- `Access-Control-Allow-Credentials` not required
- Do not use wildcard `*` for authenticated API origins

## 8) CSRF in This Design

CSRF mainly applies to cookie-auth flows where browser auto-attaches auth cookies.

In this bearer-token model:

- Browser does not auto-send `Authorization` from another site
- Classic CSRF risk is greatly reduced

Still required:

- Keep OAuth `state` validation strict (prevents login CSRF/account mix-up)
- Validate callback origins and redirect targets

## 9) XSS Mitigation Baseline (High Value, Low Effort)

- Use a strict Content Security Policy (avoid unsafe inline/eval)
- Avoid raw HTML injection (`v-html`) unless sanitized
- Keep dependencies minimal and updated
- Sanitize/encode user-provided content before rendering
- Add security headers where possible on backend responses

Note: XSS can still perform actions as user even if token exfiltration is blocked, so prevention is primary.

## 10) Provider-Specific Notes

- Register exact callback URLs in Google and Facebook developer consoles
- Keep provider scopes minimal
- Normalize provider identity to internal user model (email/provider subject)
- Prefer keeping provider access tokens server-side unless frontend directly needs provider APIs

## 11) Security Limits to Accept Explicitly

- This setup is secure enough for many community apps, but not equivalent to same-origin enterprise setups
- Long sessions increase convenience and adoption but also increase exposure window for compromised devices/tokens
- Revocation and refresh-rotation strategy is essential to control that risk

## 12) Suggested API Contract (next implementation step)

- `GET /auth/google/start`
- `GET /auth/facebook/start`
- `GET /auth/{provider}/callback`
- `POST /auth/refresh`
- `POST /auth/logout` (current device)
- `POST /auth/logout-all` (all devices)
- `GET /me`

Response payloads should include access token metadata (`exp`) and minimal user profile for client bootstrapping.

## 13) Pending Choices Resolved (for implementation)

### Identity data kept from providers

- Keep only what is needed for account identity:
  - provider (`google` or `facebook`)
  - provider subject/user id
  - verified email
- Ignore non-essential profile data for now.

### Why token persistence is needed at all

If you want reliable logout, session invalidation, and long-lived sessions, some server-side persistence is needed.

Without persistence (fully stateless JWT-only):

- No reliable logout-all-devices
- No refresh token revocation/reuse detection
- Stolen refresh token can remain valid until expiry

With persistence:

- Revoke sessions per device or globally
- Detect refresh token replay and cut off token family
- Maintain safer long-lived sessions

### Token persistence options (Axum + PostgreSQL)

1) Stateless access + DB-backed refresh tokens (recommended)

- Access token: signed JWT, short-lived, no DB lookup on normal API calls
- Refresh token: opaque random string, stored hashed in PostgreSQL
- Rotation on each refresh, with session metadata (created, last_used, expires, revoked)

Why this is the best fit:

- Good security/UX balance for mobile
- Scales well (DB hit only on refresh)
- Supports logout and security events cleanly

2) Fully stateful sessions in DB

- Every API call checks DB session id

Pros: easy revocation semantics
Cons: more DB load and latency on every request

3) Fully stateless JWT (access + refresh)

- No DB storage for tokens

Pros: simplest infra
Cons: weakest control over revocation and incident response

### Session duration policy for this project

Requested: one-week session with renewal.

Recommended implementation of that intent:

- Access token TTL: `15 minutes`
- Refresh token sliding TTL: `7 days`
- Refresh token absolute max lifetime: `30 days`
- Sliding renewal happens on refresh, not on every API call

Reasoning:

- Same user-visible behavior (users stay signed in)
- Lower DB write pressure than renewing on every access
- Better replay/risk control with rotation points

### OAuth callback URL before production URL exists

- During design and local dev, use environment-based callback origins
- In provider consoles, register local/dev redirect URI now (if needed)
- After first deployment, add production Leapcell callback URI and keep both environments separated by env vars

### Initial implementation decision set

- Framework/runtime: Rust + Axum
- Database: PostgreSQL
- Auth model: bearer access JWT + rotating DB-backed refresh tokens
- Frontend token usage: `Authorization: Bearer <token>`
- Storage compromise: access token in memory, refresh token in `localStorage`
- Scope from providers: identity email only (+ provider subject id for stable identity)
