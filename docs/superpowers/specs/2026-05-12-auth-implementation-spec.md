# Auth Implementation Spec (v1)

## Stack
- Backend: Rust `axum`
- DB: PostgreSQL
- Auth model: OAuth (Google, Facebook) -> app JWT + rotating refresh session

## Identity Model
- Persist user identity fields only:
  - `provider` (`google` | `facebook`)
  - `provider_subject` (stable provider user id)
  - `email` (normalized lowercase)

## Tokens
- Access token: JWT, `RS256` (or `EdDSA`), TTL `15m`
- Refresh token: opaque random `256-bit`, stored hashed in DB
- Refresh rotation: mandatory on every `/auth/refresh`

## Session Policy
- Sliding session window: `7d`
- Absolute session max: `30d`
- Renewal trigger: refresh only (not every API call)

## Storage (Frontend)
- Access token: memory only
- Refresh token: `localStorage`
- API auth: `Authorization: Bearer <access_token>`

## CORS
- Allow origin: `https://local-technique.github.io`
- Allow headers: `Authorization, Content-Type`
- Allow methods: `GET, POST, PUT, PATCH, DELETE, OPTIONS`
- `Vary: Origin`

## OAuth Endpoints
- `GET /auth/google/start`
- `GET /auth/facebook/start`
- `GET /auth/{provider}/callback`

## Auth Endpoints
- `POST /auth/refresh`
- `POST /auth/logout` (current session)
- `POST /auth/logout-all` (all sessions for user)
- `GET /me`

## Security Requirements
- OAuth `state` required and validated
- PKCE `S256` required
- Refresh token reuse detection -> revoke token family
- Rate limit auth endpoints
- CSP on frontend (no unsafe inline/eval)

## Database (minimum)
- `users(id, email, created_at, updated_at)`
- `user_identities(id, user_id, provider, provider_subject, email, unique(provider, provider_subject))`
- `auth_sessions(id, user_id, refresh_token_hash, created_at, last_used_at, expires_at, absolute_expires_at, revoked_at, replaced_by_session_id, user_agent, ip)`

## Env Vars
- `APP_BASE_URL`
- `FRONTEND_ORIGIN`
- `DATABASE_URL`
- `JWT_PRIVATE_KEY_PEM` / `JWT_PUBLIC_KEY_PEM`
- `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`
- `FACEBOOK_CLIENT_ID`, `FACEBOOK_CLIENT_SECRET`
