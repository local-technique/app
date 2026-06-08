# Projects Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Projects feature end-to-end from the approved PRD/design: database, backend API, frontend pages, navigation, i18n, and tests.

**Architecture:** Projects are a dedicated sibling feature to incidents and maintenances. The backend adds a `projects` module with repository/service/http layers and a migration; the frontend adds `front/src/projects` with repository, utility, listing, detail, and form files.

**Tech Stack:** Rust 2024, Axum, SQLx/PostgreSQL, Vue 3, Vue Router, vue-i18n, Vitest, vue-tsc.

**Commit Policy:** Do not commit. The user explicitly requested implementation without commits.

---

## File Map

- Create `back/migrations/0010_projects.sql`: project tables and indexes.
- Create `back/src/projects/mod.rs`: module exports.
- Create `back/src/projects/model.rs`: API DTOs and project status enum helpers.
- Create `back/src/projects/repository.rs`: SQL list/detail/edit/save/delete/translation functions.
- Create `back/src/projects/service.rs`: locale handling, validation, and unit tests.
- Create `back/src/projects/http.rs`: Axum handlers and role checks.
- Modify `back/src/main.rs`: add `projects` module.
- Modify `back/src/app/router.rs`: add `/projects` routes.
- Create `front/src/projects/types.ts`: frontend models and payloads.
- Create `front/src/projects/utils.ts`: project grouping, display status, date labels, markdown renderer, search.
- Create `front/src/projects/utils.test.ts`: pure utility tests.
- Create `front/src/projects/repositories/projectsRepository.ts`: repository interface.
- Create `front/src/projects/repositories/apiProjectsRepository.ts`: API mapping and calls.
- Create `front/src/projects/ListingPage.vue`: three-section projects listing.
- Create `front/src/projects/DetailPage.vue`: project detail page.
- Create `front/src/projects/FormPage.vue`: create/edit page.
- Create `front/src/projects/ListingPage.test.ts`: listing behavior tests.
- Create `front/src/projects/DetailPage.test.ts`: detail behavior tests.
- Modify `front/src/router/index.ts`: add project routes and guards.
- Modify `front/src/common/i18n.ts`: add EN/FR project labels.
- Modify `front/src/common/components/SidebarNav.vue`: add Projects link.
- Modify `front/src/common/components/MobileBottomNav.vue`: add Projects icon/link.
- Modify nav tests as required by assertions.

## Task 1: Backend Schema

**Files:**
- Create: `back/migrations/0010_projects.sql`

- [ ] **Step 1: Add projects migration**

Create `projects` and `project_i18n` with category FK, locale FK, status constraint, audit columns, and lookup indexes.

- [ ] **Step 2: Verify migration compiles through backend build later**

Run during final backend checks: `cargo build --all-features` from `back/`.

## Task 2: Backend Module

**Files:**
- Create: `back/src/projects/mod.rs`
- Create: `back/src/projects/model.rs`
- Create: `back/src/projects/repository.rs`
- Create: `back/src/projects/service.rs`
- Create: `back/src/projects/http.rs`
- Modify: `back/src/main.rs`
- Modify: `back/src/app/router.rs`

- [ ] **Step 1: Add service tests first**

Add unit tests in `projects/service.rs` for required fields, invalid status, valid status, date ordering, and translation field validation.

- [ ] **Step 2: Implement DTOs and validation**

Define list/detail/edit/save/translation DTOs and project validation helpers.

- [ ] **Step 3: Implement repository SQL**

Implement list/search, detail, edit data with fallback fields, save upsert, delete, list translations, and replace translations.

- [ ] **Step 4: Implement HTTP handlers**

Use project role checks: read roles for list/detail, admin/board for create/edit, admin for delete/translations.

- [ ] **Step 5: Register routes**

Add `projects` module to `main.rs` and `/projects` routes to `app/router.rs`.

## Task 3: Frontend Utilities And API

**Files:**
- Create: `front/src/projects/types.ts`
- Create: `front/src/projects/utils.ts`
- Create: `front/src/projects/utils.test.ts`
- Create: `front/src/projects/repositories/projectsRepository.ts`
- Create: `front/src/projects/repositories/apiProjectsRepository.ts`

- [ ] **Step 1: Write utility tests first**

Cover ongoing/to-come/finished classification, date-less ongoing fallback, missing date labels, markdown escaping/link rendering, and query matching.

- [ ] **Step 2: Implement project types and utilities**

Add project status derivation, date label formatting, grouping, safe markdown renderer, and search matching.

- [ ] **Step 3: Implement API repository**

Map backend snake_case DTOs to camelCase project models and implement list/byId/editData/save/delete.

## Task 4: Frontend Pages, Routes, Nav, I18n

**Files:**
- Create: `front/src/projects/ListingPage.vue`
- Create: `front/src/projects/DetailPage.vue`
- Create: `front/src/projects/FormPage.vue`
- Create: `front/src/projects/ListingPage.test.ts`
- Create: `front/src/projects/DetailPage.test.ts`
- Modify: `front/src/router/index.ts`
- Modify: `front/src/common/i18n.ts`
- Modify: `front/src/common/components/SidebarNav.vue`
- Modify: `front/src/common/components/MobileBottomNav.vue`
- Modify: `front/src/common/components/SidebarNav.test.ts`
- Modify: `front/src/common/components/MobileBottomNav.test.ts`

- [ ] **Step 1: Add page tests first**

Test listing section headings/statuses and detail description/attachments/audit/actions.

- [ ] **Step 2: Implement pages**

Build listing, detail, and form pages following event/incident page patterns.

- [ ] **Step 3: Add routes and navigation**

Register project routes and show Projects in desktop/mobile navigation.

- [ ] **Step 4: Add i18n labels**

Add EN/FR labels required by the PRD and reuse existing generic labels.

## Task 5: Verification

**Files:**
- No new files.

- [ ] **Step 1: Run frontend checks**

From `front/`, run `npm run lint`, `npm run test`, and `npm run build`. Expected: all pass.

- [ ] **Step 2: Run backend checks**

From `back/`, run `cargo clippy --all-features -- -D warnings`, `cargo test --all-features`, and `cargo build --all-features`. Expected: all pass.

- [ ] **Step 3: Review git status**

Run `git status --short` and report only files changed for this implementation plus any pre-existing unrelated changes.

## Self-Review

- Spec coverage: backend schema/API/permissions/validation/search, frontend listing/detail/form/routes/nav/i18n, tests, and CI-equivalent checks are covered.
- Placeholder scan: no placeholders remain.
- Type consistency: backend uses `code` for user-facing ID and frontend maps it to `id`; status values are `waiting` and `ongoing`; derived display statuses are `waiting`, `ongoing`, and `finished`.
