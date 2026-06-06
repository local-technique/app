# Incident Timeline Visual Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Render incident timeline entries as a vertical line-left timeline and support pending entries with no date.

**Architecture:** Update backend incident timeline persistence/API to allow nullable `at_utc` and order pending entries first. Update frontend incident types, repositories, forms, view model, and detail page to render dated entries with a `CircleCheck` icon and pending entries with an orange badge.

**Tech Stack:** Rust, SQLx, Axum, PostgreSQL migrations, Vue 3, Vue Router, Vue I18n, Vitest.

---

### Task 1: Nullable Timeline Dates

**Files:**
- Create: `back/migrations/0009_nullable_incident_timeline_dates.sql`
- Modify: `back/src/incidents/model.rs`
- Modify: `back/src/incidents/repository.rs`
- Modify: `back/src/incidents/service.rs`
- Modify: `front/src/incidents/types.ts`
- Modify: `front/src/incidents/repositories/apiIncidentsRepository.ts`
- Modify: `front/src/incidents/repositories/mockIncidentsRepository.ts`
- Modify: `front/src/incidents/FormPage.vue`

- [ ] Allow `incident_timeline.at_utc` to be nullable.
- [ ] Accept and return nullable timeline dates in detail/edit/save contracts.
- [ ] Order incident timelines by `at_utc DESC NULLS FIRST, sort_order ASC`.
- [ ] Let incident forms save blank timeline date as `null`.

### Task 2: Timeline Detail Rendering

**Files:**
- Modify: `front/src/incidents/utils.ts`
- Modify: `front/src/incidents/DetailPage.vue`
- Modify: `front/src/incidents/DetailPage.test.ts`

- [ ] Add pending label and nullable date support to the view model.
- [ ] Render line-left timeline layout.
- [ ] Show `CircleCheck` before dated entry titles.
- [ ] Show orange `Pending` badge for entries with no date.
- [ ] Use compact card styling matching existing listing cards.

### Task 3: Verification

**Files:**
- All touched frontend/backend files

- [ ] Run `npm run lint`, `npm run test`, `npm run build` in `front/`.
- [ ] Run `cargo clippy --all-features -- -D warnings`, `cargo test --all-features`, `cargo build --all-features` in `back/`.
