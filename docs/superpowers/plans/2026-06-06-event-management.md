# Event Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the incident and maintenance event management PRD end to end: role changes, categories, event create/edit/delete, audit data, and frontend UI.

**Architecture:** Follow the existing Rust Axum backend module pattern and Vue frontend feature-module pattern. Add shared category persistence/API, extend incident and maintenance APIs with role-aware partial-locale save/edit-data/audit support, and add dedicated frontend pages for event forms and category administration.

**Tech Stack:** Rust, Axum, SQLx, PostgreSQL migrations, Vue 3, Vue Router, Vue I18n, Vitest, Vite.

---

### Task 1: Roles And Authorization

**Files:**
- Modify: `back/src/common/role.rs`
- Modify: `back/src/common/auth.rs`
- Modify: `back/src/admin/model.rs`
- Modify: `front/src/auth/session.ts`
- Modify: `front/src/router/index.ts`
- Test: existing frontend/backend tests plus targeted role tests where practical

- [ ] Add `CO_OWNERSHIP_BOARD` to the role catalog and assignable roles.
- [ ] Add OR-role authorization helper for backend principals.
- [ ] Add frontend OR-role route guard support.
- [ ] Update read routes to allow `ADMIN`, `CO_OWNER`, or `CO_OWNERSHIP_BOARD`.
- [ ] Update create/edit routes to require `CO_OWNERSHIP_BOARD`.

### Task 2: Category Persistence And Backend API

**Files:**
- Create: `back/migrations/0008_event_management.sql`
- Create: `back/src/categories/mod.rs`
- Create: `back/src/categories/model.rs`
- Create: `back/src/categories/repository.rs`
- Create: `back/src/categories/service.rs`
- Create: `back/src/categories/http.rs`
- Modify: `back/src/app/router.rs`
- Modify: `back/src/main.rs`

- [ ] Add category and category_i18n tables.
- [ ] Migrate current incident and maintenance category codes into categories.
- [ ] Add category list and admin CRUD endpoints.
- [ ] Enforce admin-only category mutation and referenced-category delete protection.

### Task 3: Event Backend Edit Data, Partial Save, Delete, Audit

**Files:**
- Modify: `back/src/incidents/model.rs`
- Modify: `back/src/incidents/repository.rs`
- Modify: `back/src/incidents/service.rs`
- Modify: `back/src/incidents/http.rs`
- Modify: `back/src/maintenances/model.rs`
- Modify: `back/src/maintenances/repository.rs`
- Modify: `back/src/maintenances/service.rs`
- Modify: `back/src/maintenances/http.rs`
- Modify: `back/src/app/router.rs`

- [ ] Add audit columns via migration and expose audit data in detail responses.
- [ ] Add edit-data responses with exact and fallback localized values.
- [ ] Add create/update payloads that write only the selected locale.
- [ ] Preserve other locales on save.
- [ ] Require `CO_OWNERSHIP_BOARD` for create/update and `ADMIN` for delete.
- [ ] Join categories for current category display data.

### Task 4: Frontend Repositories, Routes, Forms, And Category Admin

**Files:**
- Modify/create files under `front/src/events`, `front/src/incidents`, `front/src/categories`, `front/src/admin`, `front/src/router`, `front/src/common`
- Test: existing Vitest tests plus focused tests for route guards, action visibility, and form helpers

- [ ] Extend event and incident repositories with category-aware detail, edit-data, save, and delete calls.
- [ ] Add category repository and admin CRUD UI.
- [ ] Add dedicated create/edit pages for maintenance events and incidents.
- [ ] Add edit locale selector and fallback helper display.
- [ ] Add incident timeline editing.
- [ ] Add create/edit/delete actions controlled by current roles.
- [ ] Add audit display and unknown-user fallback.

### Task 5: Verification

**Files:**
- All touched files

- [ ] Run `npm run lint` in `front/`.
- [ ] Run `npm run test` in `front/`.
- [ ] Run `npm run build` in `front/`.
- [ ] Run `cargo clippy --all-features -- -D warnings` in `back/`.
- [ ] Run `cargo test --all-features` in `back/`.
- [ ] Run `cargo build --all-features` in `back/`.
- [ ] Fix all failures introduced by this work.
