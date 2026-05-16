# local-technique

Local-technique is a lightweight demo web app for co-owners to track building events, maintenance, and incidents with bilingual support and modern responsive navigation.

## Backend auth configuration

For split hosting (GitHub Pages frontend + separate backend), configure both frontend values:

- `FRONTEND_ORIGIN`: browser origin allowed by CORS and refresh origin checks (for this app: `https://local-technique.github.io`).
- `FRONTEND_BASE_URL`: full frontend base URL used for post-OAuth callback redirects (for this app: `https://local-technique.github.io/app`).

## Persistent auth model (cross-site hosting)

The app uses rotating refresh tokens returned in JSON (not cookie-dependent) to stay logged in across reloads in the same browser tab/session when frontend/backend are on different sites.

- `POST /auth/exchange` returns `access_token` + `refresh_token`.
- Frontend stores refresh token in `sessionStorage` and refreshes via `POST /auth/refresh`.
- Backend stores hashed refresh tokens, rotates on each refresh, and detects token reuse.
- On reuse detection, backend revokes that session and its access tokens.

Security hardening in frontend:

- Strict CSP in `front/index.html` (script-src `self`, no inline scripts).
- Theme bootstrap moved to `front/public/theme-init.js` to avoid inline script exceptions.
- `img-src` constrained to `'self' data:` to reduce exfiltration channels.
- Development runs through Vite currently require `'unsafe-eval'` in `script-src`; keep that token in dev and remove it in production header-level CSP when serving built assets.

Note: CSP is currently enforced through a meta tag in `front/index.html`. For stronger protection, also set the same CSP as an HTTP response header at the hosting/CDN layer when possible.

## Oauth clients configuration

* https://console.cloud.google.com
