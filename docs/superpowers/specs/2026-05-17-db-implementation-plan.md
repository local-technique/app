# Database-backed Backend Implementation Plan (Domain-First Structure)

## Scope and intent

- Implement PostgreSQL persistence with `sqlx` (Postgres flavor).
- Use SQLx migrations to initialize schema idempotently.
- Replace in-memory auth/session handling with DB-backed persistence.
- Add domain CRUD foundations for incidents and maintenances/events.
- Keep API responses aligned with current frontend read needs.
- Prepare clean extension points for creation/edition/deletion and role-specific operations.

## Final architecture decision

Code is structured by domain (not by technical layers), with a public shared common module.

```text
back/src/
  main.rs
  config.rs
  app/
    mod.rs
    router.rs
    state.rs
  common/
    mod.rs                # pub mod db; pub mod error; pub mod i18n; pub mod pagination; ...
    db.rs
    error.rs
    i18n.rs
    authz.rs
  auth/
    mod.rs
    http.rs               # handlers
    service.rs            # oauth + session logic
    repository.rs         # sqlx queries for users/sessions
    model.rs              # request/response/domain structs
  incidents/
    mod.rs
    http.rs
    service.rs
    repository.rs
    model.rs
  maintenances/
    mod.rs
    http.rs
    service.rs
    repository.rs
    model.rs
  translations/
    mod.rs
    http.rs               # admin translation operations
    service.rs
    repository.rs
    model.rs
```

Notes:
- Each domain exposes a `pub mod` root and explicit `pub use` only for intended boundaries.
- `common/mod.rs` is public and reusable across domains (`pub mod ...`).
- Query code stays local to domain repositories, avoiding a global technical repository package.

## Migration plan (SQLx)

1. `0001_enable_extensions.sql`
   - `CREATE EXTENSION IF NOT EXISTS citext;`

2. `0002_auth_and_locales.sql`
   - `users`, `auth_sessions`, `locales`
   - indexes and constraints

3. `0003_incidents.sql`
   - `incidents`, `incident_i18n`, `incident_timeline`, `incident_timeline_i18n`

4. `0004_maintenances.sql`
   - `maintenances`, `maintenance_i18n`

5. `0005_ui_translations.sql`
   - `translation_keys`, `translation_values`

6. `0006_seed_initial_data.sql`
   - seed locales
   - seed existing incidents/maintenances and i18n values from current mock data snapshot
   - seed base UI keys and `en`/`fr` values

All migrations are idempotently applied by sqlx migration tracking.

## Runtime initialization

- Add `DATABASE_URL` and `ADMIN_EMAILS` in config.
- Build `PgPool` on startup.
- Run `sqlx::migrate!("./migrations").run(&pool).await` before serving routes.
- Store pool in `AppState` and pass into domain services.

## Domain-by-domain delivery phases

### Phase A - foundation

- Add sqlx dependencies and feature flags.
- Add DB pool + migration bootstrap.
- Implement `common/error.rs` for unified API errors.
- Keep existing auth routes compiling while persistence is introduced.

### Phase B - auth persistence replacement

- Replace in-memory session maps with DB-backed `auth_sessions` operations.
- On login callback:
  - upsert/find user by `(email, provider)`
  - assign `ADMIN` role when email matches `ADMIN_EMAILS` and role is absent
  - create/update session with refresh token hash rotation data
- Update `/me` to include roles.
- Keep current OAuth flow and token behavior unchanged externally.

### Phase C - read APIs for current frontend

- Incidents read endpoints (list/detail) using locale-priority fallback query.
- Maintenances read endpoints (list/detail) using same fallback behavior.
- Response shape matches frontend expectations (`title`, `shortDescription`, `longDescription`, etc.).

### Phase D - admin translation operations

- Incident translation matrix read/update.
- Maintenance translation matrix read/update.
- Generic key translation matrix read/update (batch operations).

### Phase E - CRUD for upcoming UI work

- Incidents create/update/delete with i18n batch payloads.
- Maintenances create/update/delete with i18n batch payloads.
- Soft validation for required fields by locale policy.

## API contract strategy

- Keep existing auth endpoints stable.
- Add domain endpoints:
  - `GET /incidents`
  - `GET /incidents/:id`
  - `GET /maintenances`
  - `GET /maintenances/:id`
- Locale behavior:
  - request locale from query/header
  - fallback chain default: requested -> `en` -> `fr`
  - fallback resolved in SQL, backend returns final user-facing values only.

CRUD and admin endpoints are introduced without breaking the current front-end.

## Data import strategy from current frontend mocks

- Snapshot and map `front/src/incidents/data/mockIncidents.ts` into incidents tables.
- Snapshot and map `front/src/events/data/mockEvents.ts` into maintenances tables.
- Keep IDs/codes stable where possible to reduce frontend friction.
- Attachments are intentionally excluded for now (future object storage feature).

## Common module requirements (explicit)

- `common/mod.rs` will expose shared submodules publicly:

```rust
pub mod authz;
pub mod db;
pub mod error;
pub mod i18n;
```

- Domain modules may import from `crate::common::*` directly.
- No circular domain dependencies; cross-domain calls go through explicit service interfaces.

## Validation and CI gates

After each implementation phase run:

- Backend:
  - `cargo clippy --all-features -- -D warnings`
  - `cargo test --all-features`
  - `cargo build --all-features`
- Frontend (after read API integration changes):
  - `npm run lint`
  - `npm run test`
  - `npm run build`

## Delivery order recommendation

1. Phase A + B in one PR (DB + auth persistence + migration baseline).
2. Phase C in second PR (frontend-compatible read model).
3. Phase D + E in third PR (admin translation and full CRUD).

This keeps risk low and lets current frontend switch to DB-backed reads early.
