# Project Status Text Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split project status into an icon/type dropdown and a required localized free-text status field.

**Architecture:** Keep `status_type` as the non-localized enum on `projects`, and store localized `status_text` in `project_i18n` beside `title` and `description`. Frontend forms save both values; listing/detail show the icon from `statusType` and the localized `statusText`, except finished projects which remain date-derived.

**Tech Stack:** Rust 2024, Axum, SQLx/PostgreSQL, Vue 3, Vue Router, vue-i18n, Vitest, vue-tsc.

**Commit Policy:** Do not commit. The user explicitly requested no commits.

---

## File Map

- Modify `back/migrations/0010_projects.sql`: rename `status` to `status_type` in the new migration before it is committed.
- Modify `back/src/projects/model.rs`: expose `status_type` and localized `status_text` in DTOs.
- Modify `back/src/projects/service.rs`: validate `status_type` and require localized `status_text`.
- Modify `back/src/projects/repository.rs`: query/search/save `status_type` and `status_text`.
- Modify `front/src/projects/types.ts`: replace `status` with `statusType` plus `statusText`.
- Modify `front/src/projects/utils.ts`: use `statusType` for classification/icon and `statusText` for display/search.
- Modify `front/src/projects/utils.test.ts`: cover status text display/search.
- Modify `front/src/projects/repositories/apiProjectsRepository.ts`: map `status_type` and `status_text`.
- Modify `front/src/projects/FormPage.vue`: dropdown for status type plus localized text input.
- Modify `front/src/projects/ListingPage.vue`: show localized status text with icon.
- Modify `front/src/projects/DetailPage.vue`: show localized status text with icon.
- Modify `front/src/projects/ListingPage.test.ts` and `DetailPage.test.ts`: assert custom status text.
- Modify `front/src/common/i18n.ts`: add label for project status text.

## Task 1: Backend Status Model

- [ ] **Step 1: Write/update failing backend tests**

Update `back/src/projects/service.rs` tests so `ProjectSaveRequest` uses `status_type`, requires `status_text`, and rejects missing/blank `status_text`.

- [ ] **Step 2: Run backend status tests red**

Run `cargo test --all-features projects::service` from `back/`. Expected: fail because code still uses `status` and does not require `status_text`.

- [ ] **Step 3: Implement backend status split**

Update migration/model/service/repository to use `status_type` and localized `status_text`.

- [ ] **Step 4: Run backend status tests green**

Run `cargo test --all-features projects::service` from `back/`. Expected: pass.

## Task 2: Frontend Status Model

- [ ] **Step 1: Write/update failing frontend tests**

Update project utility and page tests to expect localized custom status text such as `Awaiting quote` and `Installing roof`, while finished projects still display generic `Finished`.

- [ ] **Step 2: Run frontend project tests red**

Run `npm run test -- src/projects/utils.test.ts src/projects/ListingPage.test.ts src/projects/DetailPage.test.ts` from `front/`. Expected: fail because frontend still uses enum display text.

- [ ] **Step 3: Implement frontend status split**

Update types, API mapping, utilities, form, listing, detail, and i18n.

- [ ] **Step 4: Run frontend project tests green**

Run `npm run test -- src/projects/utils.test.ts src/projects/ListingPage.test.ts src/projects/DetailPage.test.ts` from `front/`. Expected: pass.

## Task 3: Verification

- [ ] **Step 1: Run frontend checks**

From `front/`, run `npm run lint`, `npm run test`, and `npm run build`. Expected: all pass.

- [ ] **Step 2: Run backend checks**

From `back/`, run `cargo clippy --all-features -- -D warnings`, `cargo test --all-features`, and `cargo build --all-features`. Expected: all pass.

## Self-Review

- Spec coverage: status type dropdown, localized status text, backend validation/search/persistence, and frontend display/form behavior are covered.
- Placeholder scan: no placeholders remain.
- Type consistency: backend uses `status_type`/`status_text`; frontend uses `statusType`/`statusText`.
