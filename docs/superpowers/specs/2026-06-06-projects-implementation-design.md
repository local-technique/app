# Projects Implementation Design

## Context

The projects PRD defines a third top-level section, `Projects`, alongside events/maintenance and incidents. Projects track co-ownership work from planning through active execution and completion.

The existing application has separate frontend and backend feature modules for incidents and maintenances. Projects will follow the same shape rather than introducing a generic content abstraction.

## Chosen Approach

Implement projects as a dedicated sibling feature to incidents and maintenances.

This approach keeps the change low-risk and aligned with existing code:

- Backend module: `back/src/projects`.
- Frontend module: `front/src/projects`.
- Routes and navigation matching the current event/incident pattern.
- Reuse existing categories, auth roles, locale fallback, audit display, and page structure.

Alternatives considered:

- A shared generic content abstraction would reduce duplication but requires a broad refactor and creates avoidable regression risk.
- Backend-first delivery would validate storage early but would leave the visible feature incomplete and increase API/UI mismatch risk.

## Backend Design

Add `back/src/projects` with:

- `model.rs` for request/response DTOs.
- `repository.rs` for SQL queries and persistence.
- `service.rs` for locale handling and validation.
- `http.rs` for Axum handlers and role checks.
- `mod.rs` to expose the module.

Register the module in `back/src/main.rs` and route it from `back/src/app/router.rs`.

### Database

Add a migration that creates:

- `projects`
- `project_i18n`

`projects` fields:

- `id UUID PRIMARY KEY`
- `code TEXT NOT NULL UNIQUE`
- `category_code TEXT NOT NULL REFERENCES event_categories(id) ON DELETE RESTRICT`
- `start_utc TIMESTAMPTZ NULL`
- `end_utc TIMESTAMPTZ NULL`
- `status_type TEXT NOT NULL CHECK (status_type IN ('waiting', 'ongoing'))`
- `last_modified_at TIMESTAMPTZ`
- `last_modified_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL`
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`

`project_i18n` fields:

- `project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE`
- `locale TEXT NOT NULL REFERENCES locales(code) ON DELETE RESTRICT`
- `field_key TEXT NOT NULL`
- `field_value TEXT NOT NULL`
- Primary key on `(project_id, locale, field_key)`

Allowed localized field keys are `title`, `description`, and `status_text`.

Project status is split into two concepts:

- `status_type` controls the icon and accepts only `waiting` or `ongoing`.
- `status_text` is required localized free text, edited with the same locale workflow as title and description.

`finished` is not stored. A project is finished when `end_utc` exists and is before the current time.

### API

Add endpoints:

- `GET /projects?locale=&q=` lists and searches projects.
- `POST /projects` creates a project.
- `GET /projects/:id?locale=` returns detail data.
- `PUT /projects/:id` updates a project.
- `DELETE /projects/:id` deletes a project.
- `GET /projects/:id/edit?locale=` returns edit data with locale fallback metadata.
- `GET /projects/:id/translations` returns translation matrix rows.
- `POST /projects/:id/translations/replace` replaces translations.

### Permissions

Use the current incident/maintenance role model:

- List/detail: `ADMIN`, `CO_OWNER`, `CO_OWNERSHIP_BOARD`.
- Create/edit: `ADMIN`, `CO_OWNERSHIP_BOARD`.
- Delete: `ADMIN` only.
- Translation matrix and replace: `ADMIN` only.

The `ADMIN`-only delete behavior is intentional and follows the existing incident backend behavior.

### Validation

Backend validation rejects:

- Missing or blank project ID.
- Missing or blank category.
- Missing or invalid status type.
- Missing or blank localized title.
- Missing or blank localized description.
- Missing or blank localized status text.
- `end_utc` before `start_utc` when both dates exist.
- Unknown category through the existing category foreign key.
- Unsupported locale or translation field key.

Search matches:

- Project code.
- Localized title.
- Localized description.
- Category code.
- Localized category label.
- Localized status text.
- Status type.

## Frontend Design

Add `front/src/projects` with:

- `types.ts` for project models, edit data, save payloads, and status section types.
- `utils.ts` for status derivation, date labels, search matching, and section grouping.
- `repositories/projectsRepository.ts` for the repository interface.
- `repositories/apiProjectsRepository.ts` for API calls and snake_case to camelCase mapping.
- `ListingPage.vue` for the project list.
- `DetailPage.vue` for project detail.
- `FormPage.vue` for create/edit.

### Routes And Navigation

Add lazy frontend routes:

- `/projects`
- `/projects/new`
- `/projects/:id`
- `/projects/:id/edit`

Route guards:

- List/detail: `ADMIN`, `CO_OWNER`, `CO_OWNERSHIP_BOARD`.
- Create/edit: `ADMIN`, `CO_OWNERSHIP_BOARD`.

Add `Projects` to the sidebar and mobile bottom navigation.

### Listing Page

The listing page has three sections in order:

- Ongoing Projects
- Projects To Come
- Finished Projects

Classification rules:

- `endUtc` before now means finished.
- Otherwise, `startUtc` before or equal to now means ongoing.
- Otherwise, stored `statusType = ongoing` means ongoing.
- Otherwise, the project is to come.

Cards show:

- Project ID and category code.
- Title linked to detail.
- Date label that handles missing dates.
- Status icon from `statusType` and localized free-text `statusText`.

Finished cards reuse the existing faded/past treatment. Ongoing and to-come cards use accent colors from the existing theme so they remain readable in light and dark modes.

### Detail Page

The detail page shows:

- Back link to projects, preserving search query when present.
- ID and category.
- Localized title.
- Date label.
- Status icon from `statusType` and localized free-text `statusText`.
- Description rendered from markdown as safe HTML.
- Attachments block, always visible, showing the existing no-attachments text.
- Last modified audit metadata.
- Edit/delete actions when the current role allows them.

Because the frontend currently has no markdown sanitizer dependency or existing markdown helper, v1 adds a small conservative project description renderer. It escapes raw HTML first, then supports only paragraphs, line breaks, unordered lists, inline code, bold, italic, and links with `http` or `https` URLs.

### Form Page

The form supports create and edit with:

- Edit language selector.
- Project ID, required on create and disabled on edit.
- Category selector, required.
- Optional start datetime.
- Optional end datetime.
- Status type selector with `waiting` and `ongoing`, used for the icon.
- Required localized status text input.
- Required title.
- Required markdown description textarea.

There is no markdown preview or toolbar in v1.

### Internationalization

Add English and French labels for:

- Projects navigation.
- Create project.
- Edit project.
- Project not found/load failed.
- Search projects.
- No projects match.
- Ongoing projects.
- Projects to come.
- Finished projects.
- Project ID.
- Project status.
- Project status text.
- Waiting.
- Ongoing.
- Finished.
- Description.
- Delete project confirmation.

Reuse existing generic labels for category, start, end, save, delete, attachments, no attachments, and last modified.

## Tests And Verification

Frontend tests:

- `projects/utils.test.ts` covers section classification, optional dates, derived finished state, stored status fallback, date labels, and search matching.
- `projects/ListingPage.test.ts` covers three sections, status visibility, empty/search states, and available actions where practical.
- `projects/DetailPage.test.ts` covers description rendering, no-attachments block, audit display, and action visibility where practical.
- Update navigation tests that assert available nav entries.

Backend tests:

- Add unit tests for pure validation helpers in `projects/service.rs`, covering required fields, status validation, date ordering, locale validation, and translation field keys.
- Keep database integration coverage to the repository compile path unless a backend database test harness already exists in the repository.

Required verification after implementation:

- Frontend: `npm run lint`, `npm run test`, `npm run build` from `front/`.
- Backend: `cargo clippy --all-features -- -D warnings`, `cargo test --all-features`, `cargo build --all-features` from `back/`.

## Out Of Scope

- Attachment upload or management.
- Markdown editor toolbar.
- Markdown preview in the edit form.
- Comments or discussions.
- Project timeline or milestones.
- Custom project categories separate from existing categories.
